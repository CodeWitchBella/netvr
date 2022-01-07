#pragma once

#include <stdint.h>

// this macro is used to mark function as exported from dynamic library
#if defined _WIN32 || defined _WIN64
#ifdef WIN_EXPORT
#define ISBL_NETVR_EXPORT __declspec(dllexport)
#else
#define ISBL_NETVR_EXPORT __declspec(dllimport)
#endif
#elif defined __linux__
#define ISBL_NETVR_EXPORT __attribute__((visibility("default")))
#else
#define ISBL_NETVR_EXPORT
#endif

#ifdef __cplusplus
extern "C"
{
#endif

    // ABI begin
    ISBL_NETVR_EXPORT int isbl_netvr_on_system_change(uint64_t xrSystem, uint64_t xrInstance, void *xrGetInstanceProcAddrPtr);
    ISBL_NETVR_EXPORT void isbl_netvr_set_logger(void (*callback)(const char *));
    ISBL_NETVR_EXPORT void *isbl_netvr_hook_get_instance_proc_addr(void *func);

    ISBL_NETVR_EXPORT int isbl_netvr_calibration_create();
    ISBL_NETVR_EXPORT void isbl_netvr_calibration_destroy(int handle);
    ISBL_NETVR_EXPORT void
    isbl_netvr_calibration_add_pair(int handle,
                                    double x1, double y1, double z1, double qx1, double qy1, double qz1, double qw1,
                                    double x2, double y2, double z2, double qx2, double qy2, double qz2, double qw2);
    // ABI end

// clean up defines
#ifndef ISBL_NETVR_EXPORT_NO_UNDEF
#undef ISBL_NETVR_EXPORT
#endif

#ifdef __cplusplus
} // end extern "C"
#endif
