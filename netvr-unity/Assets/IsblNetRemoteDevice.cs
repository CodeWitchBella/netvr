using System.Collections;
using System.Collections.Generic;
using System.Threading.Tasks;
using GLTFast;
using UnityEngine;

public class IsblNetRemoteDevice : MonoBehaviour
{
    public int Id = 0;
    public byte Type = 0;
    byte _loadedType = 0;
    public bool Visited = false;

    void Update()
    {
        if (_loadedType != Type)
        {
            _loadedType = Type;
            _ = LoadModel();
        }
    }

    async Task LoadModel()
    {
        // TODO: deduplicate with IsblTrackedPoseDriver (probably create dedicated component)
        ClearChildren();
        var info = IsblDeviceModel.GetInfo(Type);
        var gltf = new GltfImport();
        var success = await gltf.Load(info.ModelPath);

        if (success && _loadedType == Type)
        {
            ClearChildren();

            gltf.InstantiateMainScene(transform);
            var model = transform.GetChild(0);
            var root = model.Find(info.RootNode);
            root.parent = transform;
            Destroy(model.gameObject);

            root.localPosition = info.Position;
            root.localEulerAngles = info.Rotation;
        }
    }

    void ClearChildren()
    {
        for (var i = 0; i < transform.childCount; ++i)
            Destroy(transform.GetChild(i).gameObject);
    }
}
