using System.Collections.Generic;
using UnityEngine;

[RequireComponent(typeof(Isbl.NetVR.NetObject))]
[RequireComponent(typeof(Collider))]
public class Grabbable : MonoBehaviour
{
    Isbl.NetVR.NetObject _netObject;
    Collider _collider;
    void Start()
    {
        _collider = GetComponent<Collider>();
        _netObject = GetComponent<Isbl.NetVR.NetObject>();
    }

    void Update()
    {
        var maxPortion = 0f;
        foreach (var dev in IsblTrackedPoseDriver.Devices)
        {
            if (!dev.NetDevice.TriggerAvailable) continue;
            if (_netObject.GrabbedBy != null && _netObject.GrabbedBy != dev.gameObject) continue;
            var contains = (_collider.ClosestPoint(dev.GrabPoint) - dev.GrabPoint).sqrMagnitude < 0.0001f;

            _netObject.GrabbedBy = dev.NetDevice.Trigger > .5f && contains ? dev.gameObject : null;
            if (_netObject.GrabbedBy)
            {
                _netObject.transform.parent = dev.transform;
                _netObject.RequestMove(dev.transform.position, dev.transform.rotation);
            }
            else
            {
                _netObject.transform.parent = null;
            }
        }
    }
}
