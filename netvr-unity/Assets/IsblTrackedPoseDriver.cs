using UnityEngine;
using GLTFast;
using System.Threading.Tasks;
using UnityEngine.XR;
using System.Collections.Generic;

public class IsblTrackedPoseDriver : MonoBehaviour
{
    public bool LeftHand = true;

    IsblXRDevice _device;

    void OnEnable()
    {
        InputDevices.deviceConnected += DeviceConnected;
        InputDevices.deviceDisconnected += DeviceDisconnected;
        Cleanup();
        InitializeIfNeeded();
    }

    void OnDisable()
    {
        InputDevices.deviceConnected -= DeviceConnected;
        InputDevices.deviceDisconnected -= DeviceDisconnected;
        Cleanup();
    }

    void DeviceDisconnected(InputDevice obj)
    {
        if (_device?.Device == obj) Cleanup();
        InitializeIfNeeded();
    }

    void DeviceConnected(InputDevice obj)
    {
        if (_device != null) return; // device already connected
        InitializeIfNeeded();
    }

    static async Task<GltfImport> LoadModel(IsblDeviceModel info, string controllerName)
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

    void Cleanup()
    {
        _device = null;
        for (var i = 0; i < transform.childCount; ++i)
            Destroy(transform.GetChild(i).gameObject);
    }

    async void InitializeIfNeeded()
    {
        if (_device != null) return; // not needed

        // get device
        var devices = new List<InputDevice>();
        if (LeftHand) InputDevices.GetDevicesAtXRNode(XRNode.LeftHand, devices);
        else InputDevices.GetDevicesAtXRNode(XRNode.RightHand, devices);
        if (devices.Count < 1) return;
        var device = new IsblXRDevice(devices[0]);
        _device = device;

        // load model
        var builder = IsblDeviceModel.GetInfo(deviceName: device.Device.name, leftHand: LeftHand);
        var gltf = await LoadModel(builder, device.Device.name);

        // check for disconnect/reconnect in the mean time
        if (device != _device) return;

        // instantiate model
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
        gameObject.transform.localPosition = _device.DevicePosition;
        gameObject.transform.localRotation = _device.DeviceRotation;
    }
}
