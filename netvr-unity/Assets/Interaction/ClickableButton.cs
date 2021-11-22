using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Events;

/// <summary>
/// Implements button which you can interact with by getting physically close
/// and pulling trigger.
/// </summary>
///
/// Invokes ButtonResponder.OnClick in parent.
public class ClickableButton : MonoBehaviour
{
    public string ButtonName;

    public float FarInteraction = .3f;
    public float CloseInteraction = .15f;
    public IsblStaticXRDevice.Button Button = IsblStaticXRDevice.Button.TriggerButton;
    public bool LocalOnly = false;

    readonly HashSet<IsblTrackedPoseDriver> _pressedHere = new();
    static readonly HashSet<ClickableButton> _buttons = new();

    void OnEnable()
    {
        IsblTrackedPoseDriver.OnDeviceDisconnected += OnDeviceDisconnected;
        _buttons.Add(this);
    }
    void OnDisable()
    {
        IsblTrackedPoseDriver.OnDeviceDisconnected -= OnDeviceDisconnected;
        _buttons.Remove(this);
    }

    void OnDeviceDisconnected(IsblTrackedPoseDriver driver) { _pressedHere.Remove(driver); }

    bool IsThisButtonClosest(IsblTrackedPoseDriver dev)
    {
        var thisDist = Vector3.Distance(dev.transform.position, transform.position);
        foreach (var button in _buttons)
        {
            if (button == this) continue;
            var thatDist = Vector3.Distance(button.transform.position, transform.position);
            if (thatDist < thisDist) return false;
        }
        return true;
    }

    void Update()
    {
        var maxPortion = 0f;
        foreach (var dev in IsblTrackedPoseDriver.Devices)
        {
            if (LocalOnly && !dev.NetDevice.IsLocal) continue;

            var dist = Vector3.Distance(dev.transform.position, transform.position);

            if (dist < FarInteraction)
            {
                if (dist < CloseInteraction) maxPortion = 1f;
                else maxPortion = 1f - (dist - CloseInteraction) / (FarInteraction - CloseInteraction);

                var val = dev.NetDevice.ReadButton(Button);
                if (!val && _pressedHere.Contains(dev))
                {
                    _pressedHere.Remove(dev);
                    if (IsThisButtonClosest(dev)) InvokeClick();
                }

                if (val && !_pressedHere.Contains(dev) && IsThisButtonClosest(dev))
                    _pressedHere.Add(dev);
            }
        }

        transform.localScale = Vector3.one * (maxPortion + 1f);
    }

    void InvokeClick()
    {
        var responder = GetComponentInParent<ButtonResponder>();
        responder?.OnClick(ButtonName);
    }
}
