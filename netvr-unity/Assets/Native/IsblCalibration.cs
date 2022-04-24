using System.Collections;
using System.Collections.Generic;
using UnityEngine;

class IsblCalibration : System.IDisposable
{
    IsblDynamicLibrary _lib;
    int _handle;

    public IsblCalibration()
    {
        _lib = UnityEngine.XR.OpenXR.OpenXRSettings.Instance.GetFeature<IsblXRFeature>().Lib;
        _handle = _lib.CalibrationCreate();
    }

    public void AddPair(Vector3 p1, Quaternion q1, Vector3 p2, Quaternion q2)
    {
        _lib.CalibrationAddPair(
            _handle,
            p1.x, p1.y, p1.z, q1.x, q1.y, q1.z, q1.w,
            p2.x, p2.y, p2.z, q2.x, q2.y, q2.z, q2.w
        );
    }

    public IsblDynamicLibrary.CalibrationComputeResult Compute()
    {
        _lib.CalibrationCompute(_handle, out var result);
        return result;
    }

    public void Dispose()
    {
        _lib.CalibrationDestroy(_handle);
        _handle = 0;
        _lib = null;
    }
}
