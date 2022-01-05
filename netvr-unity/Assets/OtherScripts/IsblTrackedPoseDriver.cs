using UnityEngine;
using GLTFast;
using System.Threading.Tasks;
using UnityEngine.XR;
using System.Collections.Generic;
using System;
using System.IO;
using System.IO.Compression;

public class IsblTrackedPoseDriver : MonoBehaviour
{
    public IsblXRDevice LocalDevice = null;

    public IsblStaticXRDevice NetDevice = new();

    public static List<IsblTrackedPoseDriver> Devices = new();
    public static Action<IsblTrackedPoseDriver> OnDeviceDisconnected;

    bool _isLocalHMD;
    GameObject _modelWrapper;

#if UNITY_EDITOR
    public class SelfPropertyAttribute : PropertyAttribute { };
    [SerializeField]
    [SelfProperty]
    public IsblTrackedPoseDriver EditorOnly;
#endif

    void OnEnable()
    {
        _isLocalHMD = GetComponent<Camera>() == Camera.main;

#if UNITY_EDITOR
        EditorOnly = this;
#endif
        CleanUp();
        InitializeModelIfNeeded();
        Devices.Add(this);

        NetDevice.DeviceInfoChanged = true; // it's a new device
    }

    void CleanUpFile()
    {
        if (_file != null)
        {
            _gzip.Close();
            _file.Close();
            _gzip.Dispose();
            _file.Dispose();
            _file = null;
            _gzip = null;
        }
    }

    void OnDisable()
    {
        CleanUpFile();
        CleanUp();
        Devices.Remove(this);
        OnDeviceDisconnected?.Invoke(this);
    }

    static async Task<GltfImport> LoadModel(IsblDeviceModel info, string controllerName)
    {
        if (info == null)
        {
            if (controllerName != "none")
                Debug.LogWarning($"Unknown controller \"{controllerName}\"");
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

    void CleanUp()
    {
        Destroy(_modelWrapper);
        _modelWrapper = null;
    }

    string _loadedDevice;
    int _loadId;
    async void InitializeModelIfNeeded()
    {
        if (_loadedDevice == NetDevice.Name || _isLocalHMD) return;

        CleanUp();
        _loadedDevice = NetDevice.Name;
        var thisLoadId = ++_loadId;

        if (NetDevice.Name.Length == 0)
        {
            CleanUp();
            return;
        }

        // load model
        var builder = IsblDeviceModel.GetInfo(deviceName: NetDevice.Name, characteristics: NetDevice.Characteristics);
        var gltf = await LoadModel(builder, NetDevice.Name);

        // check for disconnect/reconnect in the mean time
        if (thisLoadId != _loadId) return;

        // instantiate model
        if (gltf != null)
        {
            if (_modelWrapper == null)
            {
                _modelWrapper = new GameObject("device model");
                _modelWrapper.transform.parent = transform;
                _modelWrapper.transform.localPosition = Vector3.zero;
                _modelWrapper.transform.localRotation = Quaternion.identity;
                _modelWrapper.transform.localScale = Vector3.one;
            }
            gltf.InstantiateMainScene(_modelWrapper.transform);
            var model = _modelWrapper.transform.GetChild(0);
            var root = model.Find(builder.RootNode);
            root.parent = _modelWrapper.transform;
            Destroy(model.gameObject);

            root.localPosition = builder.Position;
            root.localEulerAngles = builder.Rotation;
            root.localScale = Vector3.one * builder.Scale;

            ConnectAxes(_modelWrapper.transform);
        }
    }

    void ConnectAxisSync(Transform child, System.Func<float> getter)
    {
        var synchronizer = child.gameObject.AddComponent<AxisSynchronizer>();
        synchronizer.ValueGetter = getter;
    }

    void ConnectTouchpoint(Transform child)
    {
        var primitive = GameObject.CreatePrimitive(PrimitiveType.Sphere);
        primitive.transform.parent = child;
        primitive.transform.localRotation = Quaternion.identity;
        primitive.transform.localPosition = Vector3.zero;
        primitive.transform.localScale = Vector3.one * 0.005f; // 5mm
        var synchronizer = child.gameObject.AddComponent<AxisSynchronizer>();
        synchronizer.VisibilityGetter = () => NetDevice.Primary2DAxisTouch;
    }

    void ConnectAxes(Transform parent)
    {
        if (parent.name == "xr_standard_trigger_pressed_value") ConnectAxisSync(parent, () => NetDevice.Trigger);
        else if (parent.name == "xr_standard_squeeze_pressed_value") ConnectAxisSync(parent, () => NetDevice.Grip);
        else if (parent.name == "xr_standard_squeeze_pressed_mirror_value") ConnectAxisSync(parent, () => NetDevice.Grip);
        else if (parent.name == "xr_standard_touchpad_pressed_value" || parent.name == "xr_standard_thumbstick_pressed_min")
            ConnectAxisSync(parent, () => NetDevice.Primary2DAxisClick ? 1 : 0);
        else if (parent.name == "menu_pressed_value") ConnectAxisSync(parent, () => NetDevice.MenuButton ? 1 : 0);
        else if (parent.name == "xr_standard_touchpad_axes_touched_value") ConnectTouchpoint(parent);
        // cSpell:ignore xaxis, yaxis
        else if (parent.name == "xr_standard_touchpad_xaxis_touched_value" || parent.name == "xr_standard_thumbstick_xaxis_pressed_value")
            ConnectAxisSync(parent, () => (NetDevice.Primary2DAxis.x + 1) * .5f);
        else if (parent.name == "xr_standard_touchpad_yaxis_touched_value" || parent.name == "xr_standard_thumbstick_yaxis_pressed_value")
            ConnectAxisSync(parent, () => 1 - (NetDevice.Primary2DAxis.y + 1) * .5f);
        else if (parent.name == "x_button_pressed_value" || parent.name == "a_button_pressed_value")
            ConnectAxisSync(parent, () => NetDevice.PrimaryButton ? 1 : 0);
        else if (parent.name == "y_button_pressed_value" || parent.name == "b_button_pressed_value")
            ConnectAxisSync(parent, () => NetDevice.SecondaryButton ? 1 : 0);

        foreach (Transform child in parent) ConnectAxes(child);
    }

    FileStream _file;
    GZipStream _gzip;
    void Update()
    {
        if (LocalDevice != null)
        {
            NetDevice.UpdateFromDevice(LocalDevice);
            if (IsblPersistentData.Instance.LogLocalData)
            {
                if (_file == null)
                {
                    _file = File.OpenWrite(Path.Combine(IsblPersistentData.DataDirectory, $"controller-{NetDevice.LocallyUniqueId}-{DateTime.UtcNow:o}.csv.gz".Replace(":", "-")));
                    _gzip = new(_file, System.IO.Compression.CompressionLevel.Optimal);
                    _gzip.Write(System.Text.Encoding.UTF8.GetBytes("iso time;timestamp;" + NetDevice.CSVHeader + "\n"));
                }
                var now = DateTime.UtcNow;
                _gzip.Write(System.Text.Encoding.UTF8.GetBytes(
                    $"{now:o};{new DateTimeOffset(now).ToUnixTimeMilliseconds()};{NetDevice.SerializeDataAsCsv()}\n"
                    ));
            }
            else { CleanUpFile(); }
        }

        // Do not track local HMD, leave that to unity so that it's setup correctly
        if (!_isLocalHMD)
        {
            InitializeModelIfNeeded();
            gameObject.transform.localPosition = NetDevice.DevicePosition;
            gameObject.transform.localRotation = NetDevice.DeviceRotation;
        }
    }
}
