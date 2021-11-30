using System.IO;
using UnityEngine;
using UnityEngine.XR;

sealed class IsblDeviceModel
{
    public enum ControllerHand
    {
        LEFT, RIGHT, BOTH
    }

    IsblDeviceModel(string name, string dir, string file, InputDeviceCharacteristics requiredCharacteristics = InputDeviceCharacteristics.None, Vector3? position = null, Vector3? rotation = null, float scale = 1f, string rootNode = "", string folder = "Controllers")
    {
        Name = name;
        RequiredCharacteristics = requiredCharacteristics;
        ModelPath = Path.Combine(Application.streamingAssetsPath, folder, dir, file);
        Position = position ?? Vector3.zero;
        Rotation = rotation ?? new Vector3(0, 180, 0);
        Scale = scale;
        RootNode = rootNode;
    }
    public Vector3 Position { get; }
    public Vector3 Rotation { get; }
    public float Scale { get; }
    public string Name { get; }
    public string ModelPath { get; }
    public InputDeviceCharacteristics RequiredCharacteristics { get; }
    public string RootNode { get; }

    static IsblDeviceModel[] _database;

    static void InitDatabase()
    {
        if (_database == null)
        {
            _database = new IsblDeviceModel[]
            {
                new IsblDeviceModel("HTC Vive Controller OpenXR", "htc-vive", "none.glb",
                    position: new Vector3(0, -0.0083999997f, 0.0544999987f),
                    rotation: new Vector3(354.950012f, 180, 0),
                    rootNode: "htc_vive_none"
                ),
                new IsblDeviceModel("Oculus Touch Controller OpenXR", "oculus-touch-v3", "left.glb",
                    requiredCharacteristics: InputDeviceCharacteristics.Left,
                    position: new Vector3(0.00630000001f,-0.00419999985f,0.0223999992f),
                    rotation: new Vector3(346.88382f, 360f - 173.581345f, 360f - 359.492523f),
                    rootNode: "root"),
                new IsblDeviceModel("Oculus Touch Controller OpenXR", "oculus-touch-v3", "right.glb",
                    requiredCharacteristics: InputDeviceCharacteristics.Right,
                    position: new Vector3(-0.0055999998f,-0.00499999989f,0.0187999997f),
                    rotation: new Vector3(346.88382f,173.581345f,359.492523f),
                    rootNode: "root"),
                new IsblDeviceModel("Head Tracking - OpenXR", "htc-vive", "headset.glb",
                    requiredCharacteristics: InputDeviceCharacteristics.HeadMounted,
                    position:new Vector3(0.109399997f,-0.0703999996f,0.0799999982f),
                    rotation:new Vector3(273.49295f,8.64589024f,350.596466f),
                    scale: 0.0254f,
                    rootNode: "RootNode (gltf orientation matrix)/RootNode (model correction matrix)/V3.obj.cleaner.gles",
                    folder: "Headsets"),
            };
        }
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
