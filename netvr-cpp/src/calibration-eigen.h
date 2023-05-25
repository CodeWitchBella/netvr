#pragma once
#include <vector>
// adapter to Eigen code to speed up the compilation in case when calibration
// does not change.
// It makes the case of compiling when Eigen-related code did not change much
// faster because we then it's only linked into final library, instead of compiling.

/**
 * @brief A pose in 3D space. Position + orientation.
 *
 */
struct Pose
{
    double x, y, z, qx, qy, qz, qw;
};

/**
 * @brief A sample of a reference and target pose.
 *
 */
struct Sample
{
    Pose ref, target;
};

/**
 * @brief Result of the calibration
 *
 */
struct CalibrationResult
{
    double tx, ty, tz;
    double rex, rey, rez;
    double rqx, rqy, rqz, rqw;
};

/**
 * @brief Computes the calibration result from a set of samples.
 *
 * @param matches
 * @return CalibrationResult
 */
CalibrationResult calibrate(const std::vector<Sample> &matches);
