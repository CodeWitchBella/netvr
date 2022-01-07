#include "netvr-internal.h"

#include <string>
#include <vector>
#include <iostream>
#include <unordered_map>
#include <memory>

#include <Eigen/Dense>

// BEGIN: copied almost verbatim from OVR space calibrator

struct Pose
{
    Eigen::Matrix3d rot;
    Eigen::Vector3d trans;

    Pose() {}
    Pose(double x, double y, double z, double qx, double qy, double qz, double qw)
    {
        Eigen::Quaternion<double> q(qx, qy, qz, qw);
        rot = q.matrix();
        trans = Eigen::Vector3d(x, y, z);
    }
    Pose(double x, double y, double z) : trans(Eigen::Vector3d(x, y, z)) {}
};

struct Sample
{
    Pose ref, target;
    bool valid;
    Sample() : valid(false) {}
    Sample(Pose ref, Pose target) : valid(true), ref(ref), target(target) {}
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

class Calibration
{
public:
    std::vector<Sample> samples;
    Eigen::Vector3d calibratedRotation, calibratedTranslation;
    CalibrationState state;
    Calibration() = default;
};

Eigen::Vector3d AxisFromRotationMatrix3(Eigen::Matrix3d rot)
{
    return Eigen::Vector3d(rot(2, 1) - rot(1, 2), rot(0, 2) - rot(2, 0), rot(1, 0) - rot(0, 1));
}

double AngleFromRotationMatrix3(Eigen::Matrix3d rot)
{
    return acos((rot(0, 0) + rot(1, 1) + rot(2, 2) - 1.0) / 2.0);
}

DSample DeltaRotationSamples(Sample s1, Sample s2)
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

Eigen::Vector3d CalibrateRotation(const std::vector<Sample> &samples)
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
    char buf[256];

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

    Eigen::Vector3d euler = rot.eulerAngles(2, 1, 0) * 180.0 / EIGEN_PI;

    return euler;
}

Eigen::Vector3d CalibrateTranslation(const std::vector<Sample> &samples)
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
    return transcm;
}

void CalibrationTick(Calibration &ctx, double time)
{

    if (ctx.samples.size() == 250 /* TODO: generalize */)
    {
        if (ctx.state == CalibrationState::Rotation)
        {
            ctx.calibratedRotation = CalibrateRotation(ctx.samples);

            // TODO: transform the following positions with the rotation?

            ctx.state = CalibrationState::Translation;
        }
        else if (ctx.state == CalibrationState::Translation)
        {
            ctx.calibratedTranslation = CalibrateTranslation(ctx.samples);

            // TODO: apply the transform

            ctx.state = CalibrationState::None;
        }

        ctx.samples.clear();
    }
}

// END: OVR Space Calibrator code

int map_counter = 0;
std::unordered_map<int, std::unique_ptr<Calibration>> map;

ISBL_NETVR_EXPORT int isbl_netvr_calibration_create()
{
    int handle = ++map_counter;
    map.emplace(handle, std::make_unique<Calibration>());
    return handle;
}

ISBL_NETVR_EXPORT void isbl_netvr_calibration_destroy(int handle)
{
    map.erase(handle);
}

ISBL_NETVR_EXPORT void
isbl_netvr_calibration_add_pair(int handle,
                                double x1, double y1, double z1, double qx1, double qy1, double qz1, double qw1,
                                double x2, double y2, double z2, double qx2, double qy2, double qz2, double qw2)
{
    if (map.find(handle) == map.end())
        return;

    auto &ctx = map[handle];
    ctx->samples.emplace_back(
        Pose{x1, y1, z1, qx1, qy1, qz1, qw1},
        Pose{x2, y2, z2, qx2, qy2, qz2, qw2});
}
