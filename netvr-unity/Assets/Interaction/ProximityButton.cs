using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class ProximityButton : MonoBehaviour
{
    public Action OnClick;
    public float FarInteraction = .5f;
    public float CloseInteraction = .2f;

    readonly HashSet<IsblTrackedPoseDriver> _disabled = new();

    void Start()
    {
        foreach (var dev in IsblTrackedPoseDriver.Devices)
        {
            var dist = Vector3.Distance(dev.transform.position, transform.position);
            if (dist < CloseInteraction) _disabled.Add(dev);
        }
    }

    void Update()
    {
        var maxPortion = 0f;
        foreach (var dev in IsblTrackedPoseDriver.Devices)
        {
            var dist = Vector3.Distance(dev.transform.position, transform.position);
            if (dist < CloseInteraction)
            {
                maxPortion = 1f;
                if (!_disabled.Contains(dev))
                {
                    OnClick?.Invoke();
                    _disabled.Add(dev);
                }
            }
            else if (dist < FarInteraction)
            {
                maxPortion = (dist - CloseInteraction) / (FarInteraction - CloseInteraction);
            }
            else
            {
                _disabled.Remove(dev);
            }
        }

        transform.localScale = Vector3.one * (maxPortion + 1f);
    }
}
