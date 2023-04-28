#![allow(non_snake_case)]

mod input;
use std::cmp::min;

use anyhow::{anyhow, Result};
pub use input::*;
use nalgebra::{
    Const, Dyn, Matrix3, OMatrix, Quaternion, Rotation3, RowVector3, UnitQuaternion, Vector3,
};
use netvr_data::{net::CalibrationSample, Vec3};

#[derive(Debug, Default, Clone, Copy)]
struct PoseF64 {
    pub position: Vector3<f64>,
    pub rotation: Matrix3<f64>,
}

impl From<netvr_data::Pose> for PoseF64 {
    fn from(val: netvr_data::Pose) -> Self {
        Self {
            position: convert_vector(val.position),
            rotation: convert_quaternion(val.orientation)
                .to_rotation_matrix()
                .into(),
        }
    }
}

fn convert_vector(p: netvr_data::Vec3) -> Vector3<f64> {
    return Vector3::new(p.x, p.y, p.z).cast::<f64>();

    Vector3::new(p.x, p.y, -p.z).cast::<f64>()
}

fn convert_quaternion(q: netvr_data::Quaternion) -> UnitQuaternion<f64> {
    return UnitQuaternion::new_unchecked(Quaternion::new(q.w, q.x, q.y, q.z)).cast::<f64>();

    let q = UnitQuaternion::new_unchecked(Quaternion::new(q.w, -q.x, -q.y, -q.z)).cast::<f64>();
    let euler = q.euler_angles();
    let q = UnitQuaternion::from_euler_angles(euler.1, euler.0, -euler.2);
    UnitQuaternion::new_unchecked(Quaternion::new(q.w, q.j, q.i, q.k))
}

#[derive(Debug, Default, Clone, Copy)]
struct SampleF64 {
    pose: PoseF64,
}

impl From<CalibrationSample> for SampleF64 {
    fn from(val: CalibrationSample) -> Self {
        Self {
            pose: val.pose.into(),
        }
    }
}

#[derive(Default, Clone, Debug, Copy)]
struct SamplePairF64 {
    reference: SampleF64,
    target: SampleF64,
}

#[derive(Default, Clone, Debug, Copy)]
pub struct DSample {
    reference: RowVector3<f64>,
    target: RowVector3<f64>,
}

fn AxisFromRotationMatrix3(rot: Matrix3<f64>) -> RowVector3<f64> {
    RowVector3::new(
        rot[(2, 1)] - rot[(1, 2)],
        rot[(0, 2)] - rot[(2, 0)],
        rot[(1, 0)] - rot[(0, 1)],
    )
}

fn AngleFromRotationMatrix3(rot: Matrix3<f64>) -> f64 {
    f64::acos((rot.diagonal().sum() - 1.0) / 2.0)
    // return acos((rot(0, 0) + rot(1, 1) + rot(2, 2) - 1.0) / 2.0);
}

fn DeltaRotationSamples(s1: SamplePairF64, s2: SamplePairF64) -> Option<DSample> {
    // Difference in rotation between samples.
    let dref = s1.reference.pose.rotation * s2.reference.pose.rotation.transpose();
    let dtarget = s1.target.pose.rotation * s2.target.pose.rotation.transpose();

    // When stuck together, the two tracked objects rotate as a pair,
    // therefore their axes of rotation must be equal between any given pair of
    // samples.
    let ds = DSample {
        reference: AxisFromRotationMatrix3(dref),
        target: AxisFromRotationMatrix3(dtarget),
    };

    // Reject samples that were too close to each other.
    let refA = AngleFromRotationMatrix3(dref);
    let targetA = AngleFromRotationMatrix3(dtarget);
    let valid =
        refA > 0.4 && targetA > 0.4 && ds.reference.norm() > 0.01 && ds.target.norm() > 0.01;

    if valid {
        Some(DSample {
            reference: ds.reference.normalize(),
            target: ds.target.normalize(),
        })
    } else {
        None
    }
}

fn CalibrateRotation(samples: &[SamplePairF64]) -> Rotation3<f64> {
    // std::vector<DSample> deltas;
    // for (size_t i = 0; i < samples.size(); i++)
    // {
    //     for (size_t j = 0; j < i; j++)
    //     {
    //         auto delta = DeltaRotationSamples(samples[i], samples[j]);
    //         if (delta.valid)
    //             deltas.push_back(delta);
    //     }
    // }

    let mut deltas = vec![];
    for i in 0..samples.len() {
        for j in 0..i {
            let delta = DeltaRotationSamples(samples[i], samples[j]);
            if let Some(delta) = delta {
                deltas.push(delta);
            }
        }
    }

    // Kabsch algorithm
    // Eigen::Vector3d refCentroid(0, 0, 0);
    let mut refCentroid = RowVector3::zeros();
    // Eigen::Vector3d targetCentroid(0, 0, 0);
    let mut targetCentroid = RowVector3::zeros();
    // Eigen::MatrixXd refPoints(deltas.size(), 3);
    let mut refPoints = OMatrix::<f64, Dyn, Const<3>>::zeros(deltas.len());
    // Eigen::MatrixXd targetPoints(deltas.size(), 3);
    let mut targetPoints = OMatrix::<f64, Dyn, Const<3>>::zeros(deltas.len());

    // for (size_t i = 0; i < deltas.size(); i++)
    // {
    //     refPoints.row(i) = deltas[i].ref;
    //     refCentroid += deltas[i].ref;
    //     targetPoints.row(i) = deltas[i].target;
    //     targetCentroid += deltas[i].target;
    // }
    for (i, delta) in deltas.iter().enumerate() {
        refPoints.set_row(i, &delta.reference);
        refCentroid += delta.reference;
        targetPoints.set_row(i, &delta.target);
        targetCentroid += deltas[i].target;
    }
    // refCentroid /= (double)deltas.size();
    // targetCentroid /= (double)deltas.size();
    refCentroid /= deltas.len() as f64;
    targetCentroid /= deltas.len() as f64;

    // for (size_t i = 0; i < deltas.size(); i++)
    // {
    //     refPoints.row(i) -= refCentroid;
    //     targetPoints.row(i) -= targetCentroid;
    // }
    for i in 0..deltas.len() {
        refPoints.set_row(i, &(refPoints.row(i) - refCentroid));
        targetPoints.set_row(i, &(targetPoints.row(i) - targetCentroid));
    }

    // auto crossCV = refPoints.transpose() * targetPoints;
    let crossCV = refPoints.transpose() * targetPoints;

    // Eigen::BDCSVD<Eigen::MatrixXd> bdcsvd;
    // auto svd = bdcsvd.compute(crossCV, Eigen::ComputeThinU |
    // Eigen::ComputeThinV);
    let svd = crossCV.svd(true, true);

    // Eigen::Matrix3d i = Eigen::Matrix3d::Identity();
    // if ((svd.matrixU() * svd.matrixV().transpose()).determinant() < 0)
    // {
    //     i(2, 2) = -1;
    // }
    let mut i = Matrix3::<f64>::identity();
    if (svd.u.unwrap() * svd.v_t.unwrap()).determinant() < 0. {
        i[(2, 2)] = -1.;
    }

    // Eigen::Matrix3d rot = svd.matrixV() * i * svd.matrixU().transpose();
    // rot.transposeInPlace();
    let rot = svd.v_t.unwrap().transpose() * i * svd.u.unwrap().transpose();
    let rot = rot.transpose();

    // Eigen::Vector3d euler = rot.eulerAngles(2, 1, 0); // * 180.0 / EIGEN_PI;
    // char buf[256];
    // return euler;

    // let (roll, pitch, yaw) = Rotation3::from_matrix(&rot).euler_angles();
    // Vector3::new(yaw, pitch, roll) // I think?
    Rotation3::from_matrix(&rot)
}

fn CalibrateTranslation(samples: &[SamplePairF64]) -> Result<RowVector3<f64>> {
    let mut deltas: Vec<(Vector3<f64>, Matrix3<f64>)> = vec![];
    for i in 0..samples.len() {
        for j in 0..i {
            // auto QAi = samples[i].ref.rot.transpose();
            let QAi = samples[i].reference.pose.rotation.transpose();
            // auto QAj = samples[j].ref.rot.transpose();
            let QAj = samples[j].reference.pose.rotation.transpose();
            // auto dQA = QAj - QAi;
            let dQA = QAj - QAi;
            // auto CA = QAj * (samples[j].ref.trans - samples[j].target.trans)
            // - QAi * (samples[i].ref.trans - samples[i].target.trans);
            let CA = QAj * (samples[j].reference.pose.position - samples[j].target.pose.position)
                - QAi * (samples[i].reference.pose.position - samples[i].target.pose.position);
            // deltas.push_back(std::make_pair(CA, dQA));
            deltas.push((CA, dQA));
            // auto QBi = samples[i].target.rot.transpose();
            let QBi = samples[i].target.pose.rotation.transpose();
            // auto QBj = samples[j].target.rot.transpose();
            let QBj = samples[j].target.pose.rotation.transpose();
            // auto dQB = QBj - QBi;
            let dQB = QBj - QBi;
            // auto CB = QBj * (samples[j].ref.trans - samples[j].target.trans)
            // - QBi * (samples[i].ref.trans - samples[i].target.trans);
            let CB = QBj * (samples[j].reference.pose.position - samples[j].target.pose.position)
                - QBi * (samples[i].reference.pose.position - samples[i].target.pose.position);
            // deltas.push_back(std::make_pair(CB, dQB));
            deltas.push((CB, dQB));
        }
    }

    // Eigen::VectorXd constants(deltas.size() * 3);
    let mut constants = OMatrix::<f64, Dyn, Const<1>>::zeros(deltas.len() * 3);
    // Eigen::MatrixXd coefficients(deltas.size() * 3, 3);
    let mut coefficients = OMatrix::<f64, Dyn, Const<3>>::zeros(deltas.len() * 3);

    // for (size_t i = 0; i < deltas.size(); i++)
    // {
    //     for (int axis = 0; axis < 3; axis++)
    //     {
    //         constants(i * 3 + axis) = deltas[i].first(axis);
    //         coefficients.row(i * 3 + axis) = deltas[i].second.row(axis);
    //     }
    // }
    for i in 0..deltas.len() {
        for axis in 0..3 {
            constants[i * 3 + axis] = deltas[i].0[axis];
            coefficients.set_row(i * 3 + axis, &deltas[i].1.row(axis));
        }
    }

    println!("samples len: {}", samples.len());
    println!("delta len: {}", deltas.len());
    println!("constants len: {}", constants.len());
    println!("coefficients len: {}", coefficients.len());
    // println!("coefficients: {:#?}", coefficients);
    // println!("{:#?}", constants);
    // Eigen::Vector3d trans = coefficients.bdcSvd(Eigen::ComputeThinU |
    // Eigen::ComputeThinV).solve(constants);
    // auto transcm = trans * 100.0;
    let trans = coefficients
        .svd(true, true)
        .solve(&constants, f64::EPSILON)
        .map_err(|err| anyhow!("{:?}", err))?;

    Ok(RowVector3::new(trans[0], trans[1], trans[2]))
}

fn match_samples(input: &CalibrationInput) -> Vec<SamplePairF64> {
    let mut matches = vec![];
    for i in 0..min(input.reference.len(), input.target.len()) {
        matches.push(SamplePairF64 {
            reference: input.reference[i].clone().into(),
            target: input.target[i].clone().into(),
        });
    }
    matches
}

pub fn calibrate(samples: &CalibrationInput) -> Result<CalibrationResult> {
    // Notes from original code:
    // - it applies rotation right when it determines it
    // - it collects SampleCount for each phase
    // - it waits at least 50ms (0.05s) between samples

    let matches = match_samples(samples);

    let rot_samples = matches.len() / 2;
    let eigen_samples = matches
        .iter()
        .take(rot_samples)
        .copied()
        .collect::<Vec<SamplePairF64>>();

    let rotation = CalibrateRotation(&eigen_samples);

    let eigen_samples = matches
        .iter()
        .skip(rot_samples)
        .map(|s| SamplePairF64 {
            reference: s.reference,
            target: SampleF64 {
                pose: PoseF64 {
                    position: rotation * s.target.pose.position,
                    rotation: rotation * s.target.pose.rotation,
                },
            },
        })
        .collect::<Vec<SamplePairF64>>();

    let translation = CalibrateTranslation(&eigen_samples)?;
    println!("Solved position: {:?}", translation);
    let rotation = UnitQuaternion::from_matrix(&rotation.into()).cast::<f32>();
    println!("Solved rotation: {:?}", rotation.euler_angles());

    Ok(CalibrationResult {
        rotation: netvr_data::Quaternion {
            x: rotation.i,
            y: rotation.j,
            z: rotation.k,
            w: rotation.w,
        },
        translation: Vec3 {
            x: translation[0] as f32,
            y: translation[1] as f32,
            z: translation[2] as f32,
        },
    })
}
