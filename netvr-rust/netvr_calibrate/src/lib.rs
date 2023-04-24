use nalgebra::{Matrix3, Vector3};

#[derive(Default, Clone, Debug)]
pub struct Pose<T> {
    pub position: Vector3<T>,
    pub rotation: Matrix3<T>,
}

#[derive(Default, Clone, Debug)]
pub struct Sample<T> {
    reference: Pose<T>,
    target: Pose<T>,
}


Vector3<f64> AxisFromRotationMatrix3(rot: Matrix<f64>)
{
    return Vector3::new(rot(2, 1) - rot(1, 2), rot(0, 2) - rot(2, 0), rot(1, 0) - rot(0, 1));
}

f64 AngleFromRotationMatrix3(rot: Matrix<f64>)
{
    return acos((rot(0, 0) + rot(1, 1) + rot(2, 2) - 1.0) / 2.0);
}


fn DeltaRotationSamples( s1: Sample<f64>,  s2: Sample<f64>) -> Option<Sample<f64>>
{
    // Difference in rotation between samples.
    let dref = s1.reference.rotation * s2.reference.rotation.transpose();
    let dtarget = s1.target.rotation * s2.target.rotation.transpose();

    // When stuck together, the two tracked objects rotate as a pair,
    // therefore their axes of rotation must be equal between any given pair of samples.
    let ds = Sample<f64>::default();
    ds.reference = AxisFromRotationMatrix3(dref);
    ds.target = AxisFromRotationMatrix3(dtarget);

    // Reject samples that were too close to each other.
    let refA = AngleFromRotationMatrix3(dref);
    let targetA = AngleFromRotationMatrix3(dtarget);
    
    ds.reference.normalize();
    ds.target.normalize();
    if refA > 0.4 && targetA > 0.4 && ds.reference.norm() > 0.01 && ds.target.norm() > 0.01 {
        Some(ds)
    } else {
        None
    }
}


fn CalibrateRotation(samples: &[Sample<f64>]) -> Vector3<f64> {
    
    
    let deltas = vec![];
    for i in  0..samples.len() {
        for j in 0..i {
            let delta = DeltaRotationSamples(samples[i], samples[j]);
            if let Some(delta) = delta {
                deltas.push(delta);
            }
        }
    }
    
    //
    // Kabsch algorithm
    //
    // Eigen::MatrixXd refPoints(deltas.size(), 3), targetPoints(deltas.size(),
    // 3); Eigen::Vector3d refCentroid(0, 0, 0), targetCentroid(0, 0, 0);
    //
    // for (size_t i = 0; i < deltas.size(); i++)
    // {
    // refPoints.row(i) = deltas[i].ref;
    // refCentroid += deltas[i].ref;
    //
    // targetPoints.row(i) = deltas[i].target;
    // targetCentroid += deltas[i].target;
    // }
    //
    // refCentroid /= (double)deltas.size();
    // targetCentroid /= (double)deltas.size();
    //
    // for (size_t i = 0; i < deltas.size(); i++)
    // {
    // refPoints.row(i) -= refCentroid;
    // targetPoints.row(i) -= targetCentroid;
    // }
    //
    // auto crossCV = refPoints.transpose() * targetPoints;
    //
    // Eigen::BDCSVD<Eigen::MatrixXd> bdcsvd;
    // auto svd = bdcsvd.compute(crossCV, Eigen::ComputeThinU |
    // Eigen::ComputeThinV);
    //
    // Eigen::Matrix3d i = Eigen::Matrix3d::Identity();
    // if ((svd.matrixU() * svd.matrixV().transpose()).determinant() < 0)
    // {
    // i(2, 2) = -1;
    // }
    //
    // Eigen::Matrix3d rot = svd.matrixV() * i * svd.matrixU().transpose();
    // rot.transposeInPlace();
    //
    // Eigen::Vector3d euler = rot.eulerAngles(2, 1, 0); // * 180.0 / EIGEN_PI;
    // char buf[256];
    // snprintf(buf, sizeof buf, "Calibrated rotation x=%.2f y=%.2f z=%.2f\n",
    // euler[0], euler[1], euler[2]); unity_log(buf);
    // return euler;
    Vector3::new(0., 0., 0.)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
