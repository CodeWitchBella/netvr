using UnityEngine;
using GLTFast;
using System.Threading.Tasks;
using UnityEngine.XR;
using System.Collections.Generic;
using System;
using System.Diagnostics.CodeAnalysis;

public class IsblTrackedPoseDriver : MonoBehaviour
{
    public XRNode Node = XRNode.LeftHand;

    public IsblXRDevice Device { get; private set; }

#if UNITY_EDITOR
    public class SelfPropertyAttribute : PropertyAttribute { };
    [SerializeField]
    [SelfProperty]
    public GameObject Hack;
#endif

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
        if (Device?.Device == obj) Cleanup();
        InitializeIfNeeded();
    }

    void DeviceConnected(InputDevice obj)
    {
        if (Device != null) return; // device already connected
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
        Device = null;
        for (var i = 0; i < transform.childCount; ++i)
            Destroy(transform.GetChild(i).gameObject);
    }

    async void InitializeIfNeeded()
    {
        if (Device != null) return; // not needed

        // get device
        var devices = new List<InputDevice>();
        InputDevices.GetDevicesAtXRNode(Node, devices);
        if (devices.Count < 1) return;
        var device = new IsblXRDevice(devices[0]);
        Device = device;

        // load model
        var builder = IsblDeviceModel.GetInfo(deviceName: device.Device.name, node: Node);
        var gltf = await LoadModel(builder, device.Device.name);

        // check for disconnect/reconnect in the mean time
        if (device != Device) return;

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
        if (Device == null) return;
        gameObject.transform.localPosition = Device.DevicePosition;
        gameObject.transform.localRotation = Device.DeviceRotation;
    }
}
