#pragma once
#include <vector>
// adapter to Eigen code to speed up the compilation in case when calibration
// does not change.
// It makes the case of compiling when Eigen-related code did not change much
// faster because we then it's only linked into final library, instead of compiling.

struct Pose
{
    double x, y, z, qx, qy, qz, qw;
};

struct Sample
{
    Pose ref, target;
};

struct CalibrationResult
{
    double tx, ty, tz;
    double rex, rey, rez;
    double rqx, rqy, rqz, rqw;
};

CalibrationResult calibrate(const std::vector<Sample> &matches);
