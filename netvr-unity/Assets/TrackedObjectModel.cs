using System.IO;
using UnityEngine;

sealed class TrackedObjectModel
{
    public enum ControllerHand
    {
        LEFT, RIGHT, BOTH
    }

    TrackedObjectModel(string name, string dir, string file, ControllerHand hand = ControllerHand.BOTH, Vector3? position = null, Vector3? rotation = null, string rootNode = "")
    {
        Name = name;
        Hand = hand;
        ModelPath = Path.Combine(Application.streamingAssetsPath, "Controllers", dir, file);
        Position = position ?? Vector3.zero;
        Rotation = rotation ?? new Vector3(0, 180, 0);
        RootNode = rootNode;
    }
    public Vector3 Position { get; }
    public Vector3 Rotation { get; }
    public string Name { get; }
    public string ModelPath { get; }
    public ControllerHand Hand { get; }
    public string RootNode { get; }

    static TrackedObjectModel[] _database;

    public static TrackedObjectModel GetInfo(string deviceName, bool leftHand)
    {
        if (_database == null)
        {
            _database = new TrackedObjectModel[]
            {
                new TrackedObjectModel("HTC Vive Controller OpenXR", "htc-vive", "none.glb",
                    position: new Vector3(0, -0.0083999997f, 0.0544999987f),
                    rotation: new Vector3(354.950012f, 180, 0),
                    rootNode: "htc_vive_none"
                ),
                new TrackedObjectModel("Oculus Touch Controller OpenXR", "oculus-touch-v3", "left.glb",
                    hand: ControllerHand.LEFT,
                    position: new Vector3(0.00630000001f,-0.00419999985f,0.0223999992f),
                    rotation: new Vector3(346.88382f, 360f - 173.581345f, 360f - 359.492523f),
                    rootNode: "root"),
                new TrackedObjectModel("Oculus Touch Controller OpenXR", "oculus-touch-v3", "right.glb",
                    hand: ControllerHand.RIGHT,
                    position: new Vector3(-0.0055999998f,-0.00499999989f,0.0187999997f),
                    rotation: new Vector3(346.88382f,173.581345f,359.492523f),
                    rootNode: "root")
            };
        }

        foreach (var info in _database)
        {
            var correctHand = info.Hand == ControllerHand.BOTH || (leftHand == (info.Hand == ControllerHand.LEFT));
            if (correctHand && deviceName == info.Name)
            {
                return info;
            }
        }
        return null;
    }
}
