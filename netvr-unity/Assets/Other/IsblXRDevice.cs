using System.Collections.Generic;
using UnityEngine;
using UnityEngine.XR;

/// <summary>
/// Wrapper around XR.InputDevice which allows me to skip iterating available
/// usages every time I access it.
/// </summary>
public class IsblXRDevice
{
    public InputDevice Device { get; }
    static int _locallyUniqueIdGenerator;
    public readonly int LocallyUniqueId;
    public string Name => Device.name;
    public InputDeviceCharacteristics Characteristics => Device.characteristics;

    public InputFeatureUsage<Quaternion>[] Quaternion { get; }
    public InputFeatureUsage<Vector3>[] Vector3 { get; }
    public InputFeatureUsage<Vector2>[] Vector2 { get; }
    public InputFeatureUsage<float>[] Float { get; }
    public InputFeatureUsage<bool>[] Bool { get; }
    public InputFeatureUsage<uint>[] Uint { get; }
    // Types bellow are not observed on Quest2 Touch Controllers nor HTC Vive Wands
    // there is no reason they should not work, but YMMV
    #region untested
    public InputFeatureUsage<Bone>[] Bone { get; }
    public InputFeatureUsage<Hand>[] Hand { get; }
    public InputFeatureUsage<byte[]>[] ByteArray { get; }
    public InputFeatureUsage<Eyes>[] Eyes { get; }
    #endregion untested

    /// <summary>Returns raw tracking state</summary>
    ///
    /// Used to find correct controller in case multiple controllers are
    /// associated with the same XRNode.
    public InputTrackingState ReadTrackingState()
    {
        InputTrackingState value;
        if (!Device.TryGetFeatureValue(CommonUsages.trackingState, out value)) throw new System.Exception("Failed to read TrackingState");
        return value;
    }

    /// <summary>
    /// Constructor. I detect and categorize available Usages upon construction
    /// </summary>
    public IsblXRDevice(InputDevice device)
    {
        LocallyUniqueId = ++_locallyUniqueIdGenerator;
        Device = device;
        var featureUsages = new List<InputFeatureUsage>();
        if (Device.TryGetFeatureUsages(featureUsages))
        {
            int quaternionCounter = 0, vector3Counter = 0, vector2Counter = 0, floatCounter = 0, boolCounter = 0, uintCounter = 0;
            int boneCounter = 0, handCounter = 0, byteArrayCounter = 0, eyesCounter = 0;
            HashSet<string> seen = new(); // ignore duplicate usages
            foreach (var usage in featureUsages)
            {
                if (seen.Contains(usage.name)) continue;
                seen.Add(usage.name);
                if (usage.type == typeof(Quaternion)) quaternionCounter++;
                else if (usage.type == typeof(Vector3)) vector3Counter++;
                else if (usage.type == typeof(Vector2)) vector2Counter++;
                else if (usage.type == typeof(float)) floatCounter++;
                else if (usage.type == typeof(bool)) boolCounter++;
                else if (usage.type == typeof(uint)) uintCounter++;
                else if (usage.type == typeof(Bone)) boneCounter++;
                else if (usage.type == typeof(Hand)) handCounter++;
                else if (usage.type == typeof(byte[])) byteArrayCounter++;
                else if (usage.type == typeof(Eyes)) eyesCounter++;
                else Debug.Log($"Unknown usage type {usage.type} with name {usage.name} on {device.name}");
            }
            seen.Clear();

            Quaternion = new InputFeatureUsage<Quaternion>[quaternionCounter];
            Vector3 = new InputFeatureUsage<Vector3>[vector3Counter];
            Vector2 = new InputFeatureUsage<Vector2>[vector2Counter];
            Float = new InputFeatureUsage<float>[floatCounter];
            Bool = new InputFeatureUsage<bool>[boolCounter];
            Uint = new InputFeatureUsage<uint>[uintCounter];
            Bone = new InputFeatureUsage<Bone>[boneCounter++];
            Hand = new InputFeatureUsage<Hand>[handCounter++];
            ByteArray = new InputFeatureUsage<byte[]>[byteArrayCounter++];
            Eyes = new InputFeatureUsage<Eyes>[eyesCounter++];

            quaternionCounter = 0; vector3Counter = 0; vector2Counter = 0; floatCounter = 0; boolCounter = 0; uintCounter = 0;
            boneCounter = 0; handCounter = 0; byteArrayCounter = 0; eyesCounter = 0;
            foreach (var usage in featureUsages)
            {
                if (seen.Contains(usage.name)) continue;
                seen.Add(usage.name);
                if (usage.type == typeof(Quaternion)) Quaternion[quaternionCounter++] = new(usage.name);
                else if (usage.type == typeof(Vector3)) Vector3[vector3Counter++] = new(usage.name);
                else if (usage.type == typeof(Vector2)) Vector2[vector2Counter++] = new(usage.name);
                else if (usage.type == typeof(float)) Float[floatCounter++] = new(usage.name);
                else if (usage.type == typeof(bool)) Bool[boolCounter++] = new(usage.name);
                else if (usage.type == typeof(uint)) Uint[uintCounter++] = new(usage.name);
                else if (usage.type == typeof(Bone)) Bone[boneCounter++] = new(usage.name);
                else if (usage.type == typeof(Hand)) Hand[handCounter++] = new(usage.name);
                else if (usage.type == typeof(byte[])) ByteArray[byteArrayCounter++] = new(usage.name);
                else if (usage.type == typeof(Eyes)) Eyes[eyesCounter++] = new(usage.name);
            }
        }
    }
}

