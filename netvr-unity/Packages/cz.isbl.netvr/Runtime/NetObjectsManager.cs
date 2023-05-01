using System.Collections.Generic;
using UnityEngine;
using System.Linq;
using System;
using UnityEngine.XR.OpenXR;


namespace Isbl.NetVR
{

    public class NetObjectManager : MonoBehaviour
    {
        readonly List<NetObject> _objects = new();

        void Start()
        {
            _objects.Clear();
            System.UInt32 i = 0;
            foreach (Transform child in transform)
            {
                var comp = child.gameObject.GetComponent<NetObject>() != null ? child.gameObject.GetComponent<NetObject>() : child.gameObject.AddComponent<NetObject>();
                _objects.Add(comp);
                comp.Id = i++;
            }
        }

        void OnEnable()
        {
            var feature = IsblXRFeature.Instance;
            if (feature == null) return;
            feature.onSessionBegin += OnSessionBegin;
            if (feature.XrSession != 0) OnSessionBegin();
        }

        void OnDisable()
        {
            var feature = IsblXRFeature.Instance;
            if (feature == null) return;
            feature.onSessionBegin -= OnSessionBegin;
        }

        void OnSessionBegin()
        {
            var feature = IsblXRFeature.Instance;
            if (feature == null) return;
            var rpc = feature.RPC;
            if (rpc == null) return;


            rpc.InitRemoteObjects(new(
                feature.XrInstance,
                feature.XrSession,
                new(new(_objects.Select(o => new Binary.Pose(
                    Convertor.Vector3(o.transform.position),
                    Convertor.Quaternion(o.transform.rotation)
                )).ToArray()))
            ));
        }

        void Update()
        {
            var feature = IsblXRFeature.Instance;
            if (feature == null || feature.XrSession == 0) return;
            var rpc = feature.RPC;
            if (rpc == null) return;

            foreach (var obj in _objects)
            {
                if (obj.JustGrabbed)
                {
                    rpc.Grab(new(
                        feature.XrInstance,
                        feature.XrSession,
                        obj.Id
                    ));
                    obj.JustGrabbed = false;
                }
                if (obj.JustReleased)
                {
                    rpc.Release(new(
                        feature.XrInstance,
                        feature.XrSession,
                        obj.Id
                    ));
                    obj.JustReleased = false;
                }

                if (obj.GrabbedBy)
                {
                    rpc.ObjectSetPose(new(
                        feature.XrInstance,
                        feature.XrSession,
                        obj.Id,
                        new Binary.Pose(
                            Convertor.Vector3(obj.transform.position),
                            Convertor.Quaternion(obj.transform.rotation)
                        )
                    ));
                }
            }

            var objects = rpc.ReadRemoteObjects(new(
                feature.XrInstance,
                feature.XrSession
            ));
            for (var i = 0; i < objects.objects.Count(); ++i)
            {
                var comp = _objects[i];
                var pose = objects.objects[i];
                comp.transform.position = Convertor.Vector3(pose.position);
                comp.transform.rotation = Convertor.Quaternion(pose.orientation);
            }
        }
    }
}
