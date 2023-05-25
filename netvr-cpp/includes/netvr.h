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

    /**
     * @brief Called when OpenXR system changes. Does not actually do anything.
     *
     * @param xrSystem
     * @param xrInstance
     * @param xrGetInstanceProcAddrPtr
     * @return int
     */
    ISBL_NETVR_FUNC int isbl_netvr_on_system_change(uint64_t xrSystem, uint64_t xrInstance, void *xrGetInstanceProcAddrPtr);

    /**
     * @brief Sets the logger so that C++ code can log to the Unity console.
     *
     * @param callback
     * @return void
     */
    ISBL_NETVR_FUNC void isbl_netvr_set_logger(void (*callback)(const char *));

    /**
     * @brief Returns a pointer to the function that the engine will call when it
     * wants to get a function pointer to an OpenXR function.
     *
     * @param func
     * @return void*
     */
    ISBL_NETVR_FUNC void *isbl_netvr_hook_get_instance_proc_addr(void *func);

    /**
     * @brief Wasm binding for working with calibration.
     *
     * @return int
     */
    ISBL_NETVR_FUNC int isbl_netvr_calibration_create();
    /**
     * @brief To be called after calibration finished so that the memory can be freed.
     *
     * @param handle
     * @return ISBL_NETVR_FUNC
     */
    ISBL_NETVR_FUNC void isbl_netvr_calibration_destroy(int handle);
    /**
     * @brief Add a pair of poses to the calibration.
     *
     * @param handle
     * @param x1
     * @param y1
     * @param z1
     * @param qx1
     * @param qy1
     * @param qz1
     * @param qw1
     * @param x2
     * @param y2
     * @param z2
     * @param qx2
     * @param qy2
     * @param qz2
     * @param qw2
     */
    ISBL_NETVR_FUNC void
    isbl_netvr_calibration_add_pair(int handle,
                                    double x1, double y1, double z1, double qx1, double qy1, double qz1, double qw1,
                                    double x2, double y2, double z2, double qx2, double qy2, double qz2, double qw2);
    /**
     * @brief Compute the calibration and write the result to the output array.
     *
     * @param handle
     * @param output
     * @return ISBL_NETVR_FUNC
     */
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
