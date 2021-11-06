using System.IO;
using UnityEngine;

sealed class TrackedObjectModel
{
    public enum ControllerHand
    {
        LEFT, RIGHT, BOTH
    }

    TrackedObjectModel(string prefix, string dir, string file, ControllerHand hand = ControllerHand.BOTH, Vector3? position = null, Vector3? rotation = null)
    {
        NamePrefix = prefix;
        Hand = hand;
        ModelPath = Path.Combine(Application.streamingAssetsPath, "Controllers", dir, file);
        Position = position ?? Vector3.zero;
        Rotation = rotation ?? new Vector3(0, 180, 0);
    }
    public Vector3 Position { get; }
    public Vector3 Rotation { get; }
    public string NamePrefix { get; }
    public string ModelPath { get; }
    public ControllerHand Hand { get; }

    static TrackedObjectModel[] _database;

    public static TrackedObjectModel GetInfo(string controllerName, bool leftHand)
    {
        if (_database == null)
        {
            _database = new TrackedObjectModel[]
            {
                new TrackedObjectModel("HTCViveControllerOpenXR", "htc-vive", "none.glb",
                    position: new Vector3(0, -0.0083999997f, 0.0544999987f),
                    rotation: new Vector3(354.950012f, 180, 0)
                ),
                new TrackedObjectModel("OculusTouchControllerOpenXR", "oculus-touch-v3", "left.glb", hand: ControllerHand.LEFT),
                new TrackedObjectModel("OculusTouchControllerOpenXR", "oculus-touch-v3", "right.glb", hand: ControllerHand.RIGHT)
            };
        }

        foreach (var info in _database)
        {
            var correctHand = info.Hand == ControllerHand.BOTH || (leftHand == (info.Hand == ControllerHand.LEFT));
            if (correctHand && controllerName.StartsWith(info.NamePrefix))
            {
                return info;
            }
        }
        return null;
    }
}
