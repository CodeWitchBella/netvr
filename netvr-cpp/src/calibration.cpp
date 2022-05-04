#include "netvr-internal.h"
#include "calibration-eigen.h"

#include <string>
#include <vector>
#include <iostream>
#include <unordered_map>
#include <memory>

struct Calibration
{
    std::vector<Sample> samples;
};

int map_counter = 0;
std::unordered_map<int, std::unique_ptr<Calibration>> map;

ISBL_NETVR_FUNC int isbl_netvr_calibration_create()
{
    int handle = ++map_counter;
    map.emplace(handle, std::make_unique<Calibration>());
    return handle;
}

ISBL_NETVR_FUNC void isbl_netvr_calibration_destroy(int handle)
{
    map.erase(handle);
}

ISBL_NETVR_FUNC void
isbl_netvr_calibration_add_pair(int handle,
                                double x1, double y1, double z1, double qx1, double qy1, double qz1, double qw1,
                                double x2, double y2, double z2, double qx2, double qy2, double qz2, double qw2)
{
    if (map.find(handle) == map.end())
        return;

    auto &ctx = map[handle];
    ctx->samples.emplace_back(Sample{
        Pose{x1, y1, z1, qx1, qy1, qz1, qw1},
        Pose{x2, y2, z2, qx2, qy2, qz2, qw2}});
}

ISBL_NETVR_FUNC void isbl_netvr_calibration_compute(int handle, double *output)
{
    if (map.find(handle) == map.end())
    {
        for (int i = 0; i < 6; ++i)
            output[i] = 0;
        return;
    }

    auto &ctx = map[handle];
    auto result = calibrate(ctx->samples);
    std::string log("isbl_netvr_calibration_compute");
    log += "\nrot euler: " + std::to_string(result.rex);
    log += " " + std::to_string(result.rey);
    log += " " + std::to_string(result.rez);
    log += "\ntranslate: " + std::to_string(result.tx);
    log += " " + std::to_string(result.ty);
    log += " " + std::to_string(result.tz);
    log += "\nrot quaternion: " + std::to_string(result.rqx);
    log += " " + std::to_string(result.rqy);
    log += " " + std::to_string(result.rqz);
    log += " " + std::to_string(result.rqw);
    unity_log(log.c_str());
    static_assert(sizeof(double) * 10 == sizeof(decltype(result)));
    memcpy(output, &result, sizeof(double) * 10);
}
