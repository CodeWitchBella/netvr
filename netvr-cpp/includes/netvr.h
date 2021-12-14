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
// ABI end

// clean up defines
#ifndef ISBL_NETVR_EXPORT_NO_UNDEF
#undef ISBL_NETVR_EXPORT
#endif

#ifdef __cplusplus
} // end extern "C"
#endif
