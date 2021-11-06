using UnityEngine;
using GLTFast;
using System.Threading.Tasks;
using UnityEngine.XR;
using System.Collections.Generic;

public class XRTrackedController : MonoBehaviour
{
    public bool LeftHand = true;

    string _initializedControllerName = "";

    async Task<GltfImport> LoadModel(TrackedObjectModel info, string controllerName)
    {
        if (info == null)
        {
            if (controllerName != "none")
                Debug.LogError(string.Format("Unknown controller \"{0}\"", controllerName));
            return null;
        }

        var gltf = new GltfImport();
        var success = await gltf.Load(info.ModelPath);

        if (!success)
        {
            Debug.LogError("Loading glTF failed!");
            return null;
        }
        return gltf;
    }

    async void InitializeIfNeeded(string currentName)
    {
        if (currentName == _initializedControllerName) return;
        _initializedControllerName = currentName;

        var builder = TrackedObjectModel.GetInfo(deviceName: currentName, leftHand: LeftHand);
        var gltf = await LoadModel(builder, currentName);

        for (var i = 0; i < transform.childCount; ++i)
            Destroy(transform.GetChild(i).gameObject);

        if (gltf != null)
        {
            gltf.InstantiateMainScene(transform);
            var model = transform.GetChild(0);
            var root = model.Find(builder.RootNode);
            root.parent = transform;
            Destroy(model.gameObject);

            root.localPosition = builder.Position;
            root.localEulerAngles = builder.Rotation;
        }
    }

    void Update()
    {
        var devices = new List<InputDevice>();
        if (LeftHand) InputDevices.GetDevicesAtXRNode(XRNode.LeftHand, devices);
        else InputDevices.GetDevicesAtXRNode(XRNode.RightHand, devices);

        if (devices.Count < 1)
        {
            InitializeIfNeeded("none");
            return;
        }
        var device = devices[0];
        InitializeIfNeeded(device.name);

        // SteamVR with Quest 2 Link: Primary2DAxis Grip GripButton Menu PrimaryButton PrimaryTouch SecondaryButton SecondaryTouch Trigger TriggerButton TriggerTouch Primary2DAxisClick Primary2DAxisTouch IsTracked TrackingState DevicePosition DeviceRotation DeviceVelocity DeviceAngularVelocity IsTracked TrackingState PointerPosition PointerRotation PointerVelocity PointerAngularVelocity
        /*var featureUsages = new List<InputFeatureUsage>();
        if (device.TryGetFeatureUsages(featureUsages))
        {
            string list = string.Join(" ", featureUsages.ConvertAll((usage) => usage.name));
            Debug.Log(list);
        }*/

        Vector3 pos;
        device.TryGetFeatureValue(new InputFeatureUsage<Vector3>("DevicePosition"), out pos);
        Quaternion rot;
        device.TryGetFeatureValue(new InputFeatureUsage<Quaternion>("DeviceRotation"), out rot);

        gameObject.transform.localPosition = pos;
        gameObject.transform.localRotation = rot;
    }
}
