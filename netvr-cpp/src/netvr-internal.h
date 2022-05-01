#pragma once
// includes/netvr.h is meant to be as useable in programs outside of netvr-cpp
// (for example future unreal engine integration)
// that means that it tries to not pollute global namespace and also that it
// defaults to signatures meant for *importing* dlls. Following two defines
// switch that behaviour around.
#ifndef WIN_EXPORT
#define WIN_EXPORT
#endif
#define ISBL_NETVR_FUNC_NO_UNDEF

#include "../includes/netvr.h"

extern void (*unity_log)(const char *);
