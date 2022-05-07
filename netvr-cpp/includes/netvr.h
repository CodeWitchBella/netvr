#pragma once

#include <stdint.h>

// this macro is used to mark function as exported from dynamic library
#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#define ISBL_NETVR_FUNC EMSCRIPTEN_KEEPALIVE
#elif defined _WIN32 || defined _WIN64
#ifdef WIN_EXPORT
#define ISBL_NETVR_FUNC __declspec(dllexport)
#else
#define ISBL_NETVR_FUNC __declspec(dllimport)
#endif
#elif defined __linux__
#define ISBL_NETVR_FUNC __attribute__((visibility("default")))
#else
#define ISBL_NETVR_FUNC
#endif

#ifdef __cplusplus
extern "C"
{
#endif

    // ABI begin
    ISBL_NETVR_FUNC int isbl_netvr_on_system_change(uint64_t xrSystem, uint64_t xrInstance, void *xrGetInstanceProcAddrPtr);
    ISBL_NETVR_FUNC void isbl_netvr_set_logger(void (*callback)(const char *));
    ISBL_NETVR_FUNC void *isbl_netvr_hook_get_instance_proc_addr(void *func);

    ISBL_NETVR_FUNC int isbl_netvr_calibration_create();
    ISBL_NETVR_FUNC void isbl_netvr_calibration_destroy(int handle);
    ISBL_NETVR_FUNC void
    isbl_netvr_calibration_add_pair(int handle,
                                    double x1, double y1, double z1, double qx1, double qy1, double qz1, double qw1,
                                    double x2, double y2, double z2, double qx2, double qy2, double qz2, double qw2);
    ISBL_NETVR_FUNC void
    isbl_netvr_calibration_compute(int handle, double *output);
    // ABI end

// clean up defines
#ifndef ISBL_NETVR_FUNC_NO_UNDEF
#undef ISBL_NETVR_FUNC
#endif

#ifdef __cplusplus
} // end extern "C"
#endif
