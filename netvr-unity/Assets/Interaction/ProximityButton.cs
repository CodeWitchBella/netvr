using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Events;

/// <summary>
/// Implements button which you can interact with by physically hiting it
/// </summary>
///
/// Invokes ButtonResponder.OnClick in parent. Also invokes OnClick which you
/// directly set in the editor.
public class ProximityButton : MonoBehaviour
{
    public UnityEvent OnClick;
    public string ButtonName;

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
                    InvokeClick();
                    _disabled.Add(dev);
                }
            }
            else if (dist < FarInteraction)
            {
                maxPortion = 1f - (dist - CloseInteraction) / (FarInteraction - CloseInteraction);
            }
            else
            {
                _disabled.Remove(dev);
            }
        }

        transform.localScale = Vector3.one * (maxPortion + 1f);
    }

    void InvokeClick()
    {
        OnClick?.Invoke();
        var responder = GetComponentInParent<ButtonResponder>();
        responder?.OnClick(ButtonName);
    }
}
