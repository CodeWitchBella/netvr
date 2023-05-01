using System.IO;
using UnityEngine;
using UnityEngine.XR;

sealed class IsblDeviceModel
{
    public enum ControllerHand
    {
        LEFT, RIGHT, BOTH
    }

    IsblDeviceModel(string name, string dir, string file, string interactionProfile, InputDeviceCharacteristics requiredCharacteristics = InputDeviceCharacteristics.None, Vector3? position = null, Vector3? rotation = null, float? scale = null, string rootNode = "", string folder = "Controllers", Vector3? positionOffset = null, Vector3? grabPoint = null)
    {
        Name = name;
        RequiredCharacteristics = requiredCharacteristics;
        ModelPath = Path.Combine(Application.streamingAssetsPath, folder, dir, file);
        InteractionProfile = interactionProfile;
        Position = position;
        PositionOffset = positionOffset ?? Vector3.zero;
        Rotation = rotation;
        Scale = scale;
        RootNode = rootNode;
        GrabPoint = grabPoint ?? Vector3.zero;
    }
    public Vector3? Position { get; }
    public Vector3 PositionOffset { get; }
    public Vector3? Rotation { get; }
    public float? Scale { get; }
    public string Name { get; }
    public string ModelPath { get; }
    public InputDeviceCharacteristics RequiredCharacteristics { get; }
    public string RootNode { get; }
    public string InteractionProfile { get; }
    public Vector3 GrabPoint { get; }

    static IsblDeviceModel[] _database;

    static void InitDatabase()
    {
        if (_database == null)
        {
            _database = new IsblDeviceModel[]
            {
                new IsblDeviceModel("HTC Vive Controller OpenXR", "htc-vive", "none.glb",
                    "/interaction_profiles/htc/vive_controller",
                    position: new Vector3(0, -0.0083999997f, 0.0544999987f),
                    rotation: new Vector3(354.950012f, 180, 0),
                    rootNode: "htc_vive_none"
                ),
                new IsblDeviceModel("Oculus Touch Controller OpenXR", "oculus-touch-v3", "left.glb",
                    "/interaction_profiles/oculus/touch_controller",
                    requiredCharacteristics: InputDeviceCharacteristics.Left,
                    position: new Vector3(0.00630000001f,-0.00419999985f,0.0223999992f),
                    rotation: new Vector3(346.88382f, 360f - 173.581345f, 360f - 359.492523f),
                    grabPoint: new Vector3(0.00810000021f,-0.058699999f,0.0759999976f),
                    rootNode: "root"),
                new IsblDeviceModel("Oculus Touch Controller OpenXR", "oculus-touch-v3", "right.glb",
                    "/interaction_profiles/oculus/touch_controller",
                    requiredCharacteristics: InputDeviceCharacteristics.Right,
                    position: new Vector3(-0.0055999998f,-0.00499999989f,0.0187999997f),
                    rotation: new Vector3(346.88382f,173.581345f,359.492523f),
                    grabPoint: new Vector3(-0.00810000021f,-0.058699999f,0.0759999976f),
                    rootNode: "root"),
                new IsblDeviceModel("Index Controller OpenXR", "valve-index", "left.glb",
                    "/interaction_profiles/valve/index_controller",
                    requiredCharacteristics: InputDeviceCharacteristics.Left,
                    positionOffset: new Vector3(0, 0, 0.0246f),
                    grabPoint: new Vector3(0.00939999986f,-0.0472000018f,0.116099998f),
                    rootNode: "valve_index_left"),
                new IsblDeviceModel("Index Controller OpenXR", "valve-index", "right.glb",
                    "/interaction_profiles/valve/index_controller",
                    requiredCharacteristics: InputDeviceCharacteristics.Right,
                    positionOffset: new Vector3(0, 0, 0.0246f),
                    grabPoint: new Vector3(-0.00939999986f,-0.0472000018f,0.116099998f),
                    rootNode: "valve_index_right"),
                new IsblDeviceModel("Head Tracking - OpenXR", "htc-vive", "headset.glb",
                    "generic_hmd",
                    requiredCharacteristics: InputDeviceCharacteristics.HeadMounted,
                    position:new Vector3(0.109399997f,-0.0703999996f,0.0799999982f),
                    rotation:new Vector3(273.49295f,8.64589024f,350.596466f),
                    scale: 0.0254f,
                    rootNode: "RootNode (gltf orientation matrix)/RootNode (model correction matrix)/V3.obj.cleaner.gles",
                    folder: "Headsets"),
            };
        }
    }

    private static bool CheckCharacteristics(InputDeviceCharacteristics required, InputDeviceCharacteristics check)
    {
        return (required & check) == required;
    }

    private static InputDeviceCharacteristics ConstructCharacteristicsForPath(string subactionPath)
    {
        var every = InputDeviceCharacteristics.TrackedDevice;
        var hand = every | InputDeviceCharacteristics.HeldInHand | InputDeviceCharacteristics.Controller;

        if (subactionPath == "/user/hand/left") return InputDeviceCharacteristics.Left | hand;
        if (subactionPath == "/user/hand/right") return InputDeviceCharacteristics.Right | hand;
        if (subactionPath == "/user/head") return InputDeviceCharacteristics.HeadMounted | every;
        return InputDeviceCharacteristics.None;
    }

    public static IsblDeviceModel GetRemoteDeviceInfo(string interactionProfile, string subactionPath)
    {
        InitDatabase();

        foreach (var info in _database)
        {
            var correctChars = (info.RequiredCharacteristics & ConstructCharacteristicsForPath(subactionPath)) == info.RequiredCharacteristics;
            if (!correctChars) continue;
            if (info.InteractionProfile == interactionProfile)
            {
                return info;
            }
        }
        return null;
    }

    public static IsblDeviceModel GetInfo(string deviceName, InputDeviceCharacteristics characteristics)
    {
        InitDatabase();

        foreach (var info in _database)
        {
            var correctChars = (info.RequiredCharacteristics & characteristics) == info.RequiredCharacteristics;
            if (correctChars && deviceName == info.Name)
            {
                return info;
            }
        }
        return null;
    }
}
