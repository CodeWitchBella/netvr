#![allow(non_snake_case)]

use nalgebra::{Const, Dyn, Matrix3, OMatrix, Rotation3, RowVector3, Vector3};

#[derive(Default, Clone, Debug, Copy)]
pub struct Pose<T: num::Float + std::clone::Clone + std::cmp::PartialEq + std::fmt::Debug + 'static>
{
    pub position: Vector3<T>,
    pub rotation: Matrix3<T>,
}

impl From<Pose<f32>> for Pose<f64> {
    fn from(val: Pose<f32>) -> Self {
        Pose {
            position: val.position.cast::<f64>(),
            rotation: val.rotation.cast::<f64>(),
        }
    }
}

#[derive(Default, Clone, Debug, Copy)]
pub struct Sample<
    T: num::Float + std::clone::Clone + std::cmp::PartialEq + std::fmt::Debug + 'static,
> {
    reference: Pose<T>,
    target: Pose<T>,
}

impl From<Sample<f32>> for Sample<f64> {
    fn from(val: Sample<f32>) -> Self {
        Sample {
            reference: val.reference.into(),
            target: val.target.into(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct CalibrationResult {
    pub translation: RowVector3<f32>,
    pub rotation: Rotation3<f32>,
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

fn DeltaRotationSamples(s1: Sample<f64>, s2: Sample<f64>) -> Option<DSample> {
    // Difference in rotation between samples.
    let dref = s1.reference.rotation * s2.reference.rotation.transpose();
    let dtarget = s1.target.rotation * s2.target.rotation.transpose();

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

fn CalibrateRotation(samples: &[Sample<f64>]) -> Rotation3<f64> {
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

fn CalibrateTranslation(samples: &[Sample<f64>]) -> RowVector3<f64> {
    let mut deltas: Vec<(Vector3<f64>, Matrix3<f64>)> = vec![];
    for i in 0..samples.len() {
        for j in 0..i {
            // auto QAi = samples[i].ref.rot.transpose();
            let QAi = samples[i].reference.rotation.transpose();
            // auto QAj = samples[j].ref.rot.transpose();
            let QAj = samples[j].reference.rotation.transpose();
            // auto dQA = QAj - QAi;
            let dQA = QAj - QAi;
            // auto CA = QAj * (samples[j].ref.trans - samples[j].target.trans)
            // - QAi * (samples[i].ref.trans - samples[i].target.trans);
            let CA = QAj * (samples[j].reference.position - samples[j].target.position)
                - QAi * (samples[i].reference.position - samples[i].target.position);
            // deltas.push_back(std::make_pair(CA, dQA));
            deltas.push((CA, dQA));
            // auto QBi = samples[i].target.rot.transpose();
            let QBi = samples[i].target.rotation.transpose();
            // auto QBj = samples[j].target.rot.transpose();
            let QBj = samples[j].target.rotation.transpose();
            // auto dQB = QBj - QBi;
            let dQB = QBj - QBi;
            // auto CB = QBj * (samples[j].ref.trans - samples[j].target.trans)
            // - QBi * (samples[i].ref.trans - samples[i].target.trans);
            let CB = QBj * (samples[j].reference.position - samples[j].target.position)
                - QBi * (samples[i].reference.position - samples[i].target.position);
            // deltas.push_back(std::make_pair(CB, dQB));
            deltas.push((CB, dQB));
        }
    }

    // Eigen::VectorXd constants(deltas.size() * 3);
    let mut constants = OMatrix::<f64, Dyn, Const<3>>::zeros(deltas.len() * 3);
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

    // Eigen::Vector3d trans = coefficients.bdcSvd(Eigen::ComputeThinU |
    // Eigen::ComputeThinV).solve(constants);
    // auto transcm = trans * 100.0;
    let trans = constants.svd(true, true);
    let trans = trans.singular_values;

    RowVector3::new(trans[0], trans[1], trans[2])
}

pub fn calibrate(matches: &[Sample<f32>]) -> CalibrationResult {
    // Notes from original code:
    // - it applies rotation right when it determines it
    // - it collects SampleCount for each phase
    // - it waits at least 50ms (0.05s) between samples

    let rot_samples = matches.len() / 2;
    let _pos_samples = matches.len() - rot_samples;
    let eigen_samples = matches
        .iter()
        .take(rot_samples)
        .map(|s| (*s).into())
        .collect::<Vec<Sample<f64>>>();

    let rotation = CalibrateRotation(&eigen_samples);
    println!("Solved rotation: {:?}", rotation);

    let eigen_samples = matches
        .iter()
        .skip(rot_samples)
        .map(|s| (*s).into())
        .collect::<Vec<Sample<f64>>>();

    let translation = CalibrateTranslation(&eigen_samples);
    println!("Solved position: {:?}", translation);

    CalibrationResult {
        rotation: rotation.cast::<f32>(),
        translation: translation.cast::<f32>(),
    }
}
