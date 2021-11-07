using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.XR;

public class IsblXRDevice
{
    public InputDevice Device { get; }

    InputFeatureUsage<Vector3> _devicePosition;
    public Vector3 DevicePosition { get { return Read(_devicePosition); } }
    InputFeatureUsage<Quaternion> _deviceRotation;
    public Quaternion DeviceRotation { get { return Read(_deviceRotation); } }
    // ADD_FEATURE step 1: two lines above this line
    public IsblXRDevice(InputDevice device)
    {
        Device = device;
        var featureUsages = new List<InputFeatureUsage>();
        if (Device.TryGetFeatureUsages(featureUsages))
        {
            foreach (var usage in featureUsages)
            {
                InitUsage(usage);
            }
        }
    }

    void InitUsage(InputFeatureUsage usage)
    {
        // SteamVR with Quest 2 Link: Primary2DAxis Grip GripButton Menu PrimaryButton PrimaryTouch SecondaryButton SecondaryTouch Trigger TriggerButton TriggerTouch Primary2DAxisClick Primary2DAxisTouch IsTracked TrackingState DevicePosition DeviceRotation DeviceVelocity DeviceAngularVelocity IsTracked TrackingState PointerPosition PointerRotation PointerVelocity PointerAngularVelocity

        if (usage.name == "DevicePosition") _devicePosition = new InputFeatureUsage<Vector3>(usage.name);
        else if (usage.name == "DeviceRotation") _deviceRotation = new InputFeatureUsage<Quaternion>(usage.name);
        // ADD_FEATURE step 2: one line above this line
    }

    Vector3 Read(InputFeatureUsage<Vector3>? usage)
    {
        var usageVal = usage ?? throw new System.Exception("Attempted reading unsupported feature");
        Vector3 value;
        Device.TryGetFeatureValue(usageVal, out value);
        return value;
    }

    Quaternion Read(InputFeatureUsage<Quaternion>? usage)
    {
        var usageVal = usage ?? throw new System.Exception("Attempted reading unsupported feature");
        Quaternion value;
        Device.TryGetFeatureValue(usageVal, out value);
        return value;
    }

    // ADD_FEATURE step 3: if new type is needed copy add new variant of Read method
}
