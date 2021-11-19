using UnityEngine;
using GLTFast;
using System.Threading.Tasks;
using UnityEngine.XR;

public class IsblTrackedPoseDriver : MonoBehaviour
{
    public IsblStaticXRDevice NetDevice = new();

    IsblXRDeviceComponent _localDriver;
    GameObject _modelWrapper;
    void Start()
    {
        _localDriver = GetComponent<IsblXRDeviceComponent>();
        if (_localDriver != null)
        {
            var net = IsblNet.Instance;
            if (net != null)
            {
                if (_localDriver.Node == XRNode.RightHand) net.NetState.Right = NetDevice;
                else if (_localDriver.Node == XRNode.LeftHand) net.NetState.Left = NetDevice;
            }
        }
    }

#if UNITY_EDITOR
    public class SelfPropertyAttribute : PropertyAttribute { };
    [SerializeField]
    [SelfProperty]
    public IsblTrackedPoseDriver EditorOnly;
#endif

    void OnEnable()
    {
#if UNITY_EDITOR
        EditorOnly = this;
#endif
        Cleanup();
        InitializeIfNeeded();
    }

    void OnDisable()
    {
        Cleanup();
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
        Destroy(_modelWrapper);
        _modelWrapper = null;
    }

    string _loadedDevice;
    int _loadId;
    async void InitializeIfNeeded()
    {
        if (_loadedDevice == NetDevice.Name) return;

        Cleanup();
        _loadedDevice = NetDevice.Name;
        var thisLoadId = ++_loadId;

        if (NetDevice.Name.Length == 0)
        {
            Cleanup();
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
        }
    }

    void Update()
    {
        // Called here so that I do not have to rely on Script Execution Order
        if (_localDriver != null) NetDevice.UpdateFromDevice(_localDriver.Device);

        InitializeIfNeeded();
        gameObject.transform.localPosition = NetDevice.DevicePosition;
        gameObject.transform.localRotation = NetDevice.DeviceRotation;
    }
}
