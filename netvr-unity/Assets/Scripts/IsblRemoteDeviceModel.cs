using UnityEngine;
using GLTFast;
using System.Threading.Tasks;
using System.Collections.Generic;
using System;
using System.IO;
using System.IO.Compression;
using System.Text.Json;

[RequireComponent(typeof(Isbl.NetVR.IsblRemoteDevice))]
public class IsblRemoteDeviceModel : MonoBehaviour
{
    Isbl.NetVR.IsblRemoteDevice NetDevice;
    GameObject _modelWrapper;

    void OnEnable()
    {
        NetDevice = GetComponent<Isbl.NetVR.IsblRemoteDevice>();
        CleanUp();
        InitializeModelIfNeeded();
    }

    void OnDisable()
    {
        CleanUp();
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
        var syntheticName = NetDevice.InteractionProfile + NetDevice.SubactionPath;
        if (_loadedDevice == syntheticName) return;

        CleanUp();
        _loadedDevice = syntheticName;
        var thisLoadId = ++_loadId;

        // load model
        var builder = IsblDeviceModel.GetRemoteDeviceInfo(NetDevice.InteractionProfile, NetDevice.SubactionPath);
        var gltf = await IsblTrackedPoseDriver.LoadModel(builder, syntheticName);

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
            model.localEulerAngles = new Vector3(0f, 180f, 0f);
            var root = model.Find(builder.RootNode);
            root.parent = _modelWrapper.transform;
            Destroy(model.gameObject);

            if (builder.Position != null) root.localPosition = builder.Position ?? Vector3.zero;
            root.localPosition += builder.PositionOffset;
            if (builder.Rotation != null) root.localEulerAngles = builder.Rotation ?? Vector3.zero;
            if (builder.Scale != null) root.localScale = Vector3.one * (builder.Scale ?? 1f);
        }
    }

    void Update()
    {
        InitializeModelIfNeeded();
    }
}
