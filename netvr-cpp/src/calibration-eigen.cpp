#include "calibration-eigen.h"
#include "netvr-internal.h"

#include <string>
#include <vector>
#include <iostream>
#include <unordered_map>
#include <memory>

#include <Eigen/Dense>

// BEGIN: copied almost verbatim from OVR space calibrator

namespace
{
    struct EigenPose
    {
        Eigen::Matrix3d rot;
        Eigen::Vector3d trans;

        EigenPose() {}
        EigenPose(const Pose &p)
        {
            Eigen::Quaternion<double> q(p.qx, p.qy, p.qz, p.qw);
            rot = q.matrix();
            trans = Eigen::Vector3d(p.x, p.y, p.z);
        }
        EigenPose(double x, double y, double z) : trans(Eigen::Vector3d(x, y, z)) {}
    };

    struct EigenSample
    {
        EigenPose ref, target;
        bool valid;
        EigenSample() : valid(false) {}
        EigenSample(Sample s) : valid(true), ref(s.ref), target(s.target) {}
    };

    struct DSample
    {
        bool valid;
        Eigen::Vector3d ref, target;
    };

    enum CalibrationState
    {
        None,
        Rotation,
        Translation
    };

    class EigenCalibration
    {
    public:
        std::vector<EigenSample> samples;
        Eigen::Vector3d calibratedRotation, calibratedTranslation;
        CalibrationState state;
        EigenCalibration() = default;
    };

    Eigen::Vector3d AxisFromRotationMatrix3(Eigen::Matrix3d rot)
    {
        return Eigen::Vector3d(rot(2, 1) - rot(1, 2), rot(0, 2) - rot(2, 0), rot(1, 0) - rot(0, 1));
    }

    double AngleFromRotationMatrix3(Eigen::Matrix3d rot)
    {
        return acos((rot(0, 0) + rot(1, 1) + rot(2, 2) - 1.0) / 2.0);
    }

    DSample DeltaRotationSamples(EigenSample s1, EigenSample s2)
    {
        // Difference in rotation between samples.
        auto dref = s1.ref.rot * s2.ref.rot.transpose();
        auto dtarget = s1.target.rot * s2.target.rot.transpose();

        // When stuck together, the two tracked objects rotate as a pair,
        // therefore their axes of rotation must be equal between any given pair of samples.
        DSample ds;
        ds.ref = AxisFromRotationMatrix3(dref);
        ds.target = AxisFromRotationMatrix3(dtarget);

        // Reject samples that were too close to each other.
        auto refA = AngleFromRotationMatrix3(dref);
        auto targetA = AngleFromRotationMatrix3(dtarget);
        ds.valid = refA > 0.4 && targetA > 0.4 && ds.ref.norm() > 0.01 && ds.target.norm() > 0.01;

        ds.ref.normalize();
        ds.target.normalize();
        return ds;
    }

    Eigen::Vector3d CalibrateRotation(const std::vector<EigenSample> &samples)
    {
        std::vector<DSample> deltas;

        for (size_t i = 0; i < samples.size(); i++)
        {
            for (size_t j = 0; j < i; j++)
            {
                auto delta = DeltaRotationSamples(samples[i], samples[j]);
                if (delta.valid)
                    deltas.push_back(delta);
            }
        }

        // Kabsch algorithm

        Eigen::MatrixXd refPoints(deltas.size(), 3), targetPoints(deltas.size(), 3);
        Eigen::Vector3d refCentroid(0, 0, 0), targetCentroid(0, 0, 0);

        for (size_t i = 0; i < deltas.size(); i++)
        {
            refPoints.row(i) = deltas[i].ref;
            refCentroid += deltas[i].ref;

            targetPoints.row(i) = deltas[i].target;
            targetCentroid += deltas[i].target;
        }

        refCentroid /= (double)deltas.size();
        targetCentroid /= (double)deltas.size();

        for (size_t i = 0; i < deltas.size(); i++)
        {
            refPoints.row(i) -= refCentroid;
            targetPoints.row(i) -= targetCentroid;
        }

        auto crossCV = refPoints.transpose() * targetPoints;

        Eigen::BDCSVD<Eigen::MatrixXd> bdcsvd;
        auto svd = bdcsvd.compute(crossCV, Eigen::ComputeThinU | Eigen::ComputeThinV);

        Eigen::Matrix3d i = Eigen::Matrix3d::Identity();
        if ((svd.matrixU() * svd.matrixV().transpose()).determinant() < 0)
        {
            i(2, 2) = -1;
        }

        Eigen::Matrix3d rot = svd.matrixV() * i * svd.matrixU().transpose();
        rot.transposeInPlace();

        Eigen::Vector3d euler = rot.eulerAngles(2, 1, 0); // * 180.0 / EIGEN_PI;
        char buf[256];
        snprintf(buf, sizeof buf, "Calibrated rotation x=%.2f y=%.2f z=%.2f\n", euler[0], euler[1], euler[2]);
        unity_log(buf);
        return euler;
    }

    Eigen::Vector3d CalibrateTranslation(const std::vector<EigenSample> &samples)
    {
        std::vector<std::pair<Eigen::Vector3d, Eigen::Matrix3d>> deltas;

        for (size_t i = 0; i < samples.size(); i++)
        {
            for (size_t j = 0; j < i; j++)
            {
                auto QAi = samples[i].ref.rot.transpose();
                auto QAj = samples[j].ref.rot.transpose();
                auto dQA = QAj - QAi;
                auto CA = QAj * (samples[j].ref.trans - samples[j].target.trans) - QAi * (samples[i].ref.trans - samples[i].target.trans);
                deltas.push_back(std::make_pair(CA, dQA));

                auto QBi = samples[i].target.rot.transpose();
                auto QBj = samples[j].target.rot.transpose();
                auto dQB = QBj - QBi;
                auto CB = QBj * (samples[j].ref.trans - samples[j].target.trans) - QBi * (samples[i].ref.trans - samples[i].target.trans);
                deltas.push_back(std::make_pair(CB, dQB));
            }
        }

        Eigen::VectorXd constants(deltas.size() * 3);
        Eigen::MatrixXd coefficients(deltas.size() * 3, 3);

        for (size_t i = 0; i < deltas.size(); i++)
        {
            for (int axis = 0; axis < 3; axis++)
            {
                constants(i * 3 + axis) = deltas[i].first(axis);
                coefficients.row(i * 3 + axis) = deltas[i].second.row(axis);
            }
        }

        Eigen::Vector3d trans = coefficients.bdcSvd(Eigen::ComputeThinU | Eigen::ComputeThinV).solve(constants);
        auto transcm = trans * 100.0;

        char buf[256];
        snprintf(buf, sizeof buf, "Calibrated translation x=%.2f y=%.2f z=%.2f\n", transcm[0], transcm[1], transcm[2]);
        unity_log(buf);
        return transcm;
    }

} // anonymous namespace

// END: OVR Space Calibrator code

CalibrationResult calibrate(const std::vector<Sample> &matches)
{
    // Notes from original code:
    // - it applies rotation right when it determines it
    // - it collects SampleCount for each phase
    // - it waits at least 20ms (0.05s) between samples
    std::vector<EigenSample> eigen_samples;
    int rot_samples = matches.size() / 2, pos_samples = matches.size() - rot_samples;
    eigen_samples.reserve(std::max(rot_samples, pos_samples));

    for (int i = 0; i < rot_samples; ++i)
    {
        eigen_samples.emplace_back(matches[i]);
    }

    Eigen::Vector3d rotation = CalibrateRotation(eigen_samples);

    // TODO: transform the following positions with the rotation?
    eigen_samples.clear();
    for (int i = 0; i < pos_samples; ++i)
    {
        eigen_samples.emplace_back(matches[rot_samples + i]);
    }

    Eigen::Vector3d translation = CalibrateTranslation(eigen_samples);

    return CalibrationResult{
        translation.x(),
        translation.y(),
        translation.z(),
        rotation.x(),
        rotation.y(),
        rotation.z(),
    };
}
