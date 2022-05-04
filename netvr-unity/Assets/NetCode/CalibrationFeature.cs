using System;
using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;
using UnityEngine;
using System.Linq;

/*
 * Messages:
 * {
 *   feature: FeatureId,
 *   action: "begin",
 *   follower: ushort, followerDevice: ushort,
 *   leader: ushort, leaderDevice: ushort,
 * } // reflected to follower and leader
 *
 * {
 *   feature: FeatureId,
 *   action: "samples",
 *   deviceId: ushort,
 *   clientId: ushort,
 *   leader: ushort,
 *   samples: [
 *     {
 *      timestamp: time since the start message received in seconds, double,
 *      position: Vector3,
 *      rotation: Quaternion,
 *     },
 *     ... possibly multiple samples if batching is implemented
 *   ]
 * } // reflected to leader
 *
 * {
 *   feature: FeatureId,
 *   action: "end",
 *   follower: ushort, followerDevice: ushort,
 *   leader: ushort, leaderDevice: ushort,
 * } // reflected to follower
 *
 * { normal configuration patch message }
 */

public class CalibrationFeature : IIsblNetFeature
{
    public const string FeatureId = "calibration";
    const double Timeout = 180.0;
    const double FinishedDeleteTimeout = 10.0;
    const int RequiredSamples = 200; // OVR SpaceCal equivalents: FAST=200, SLOW=500, VERY_SLOW=1000
    const int SampleBatchSize = 10;

    struct RemoteCalibration
    {
        public ushort LeaderId, LocalDeviceId;
        public double StartTimeStamp;
        public List<Sample> SampleBatch;
    }

    public struct Sample
    {
        [JsonConverter(typeof(Isbl.Json.QuaternionAsEulerConverter))]
        [JsonInclude]
        [JsonPropertyName("rotation")]
        public Quaternion Rotation;

        [JsonInclude]
        [JsonPropertyName("timestamp")]
        public double Timestamp;

        [JsonConverter(typeof(Isbl.Json.Vector3Converter))]
        [JsonInclude]
        [JsonPropertyName("position")]
        public Vector3 Position;
    }

    class LocalCalibration
    {
        public ushort FollowerDeviceId, LocalDeviceId, FollowerId;
        public double StartTimeStamp;
        public double FinishedTimeStamp = 0;

        public List<Sample> LeaderSamples = new(), FollowerSamples = new();
    }

    List<LocalCalibration> _calibrationsLocal;
    List<RemoteCalibration> _calibrationsRemote;

    /**
     * Deserializes JsonObject to T. Adapted from https://stackoverflow.com/a/59047063
     */
    static T JsonElementToObject<T>(JsonElement element, JsonSerializerOptions options = null)
    {
        var bufferWriter = new System.Buffers.ArrayBufferWriter<byte>();
        using (var writer = new Utf8JsonWriter(bufferWriter))
            element.WriteTo(writer);
        return JsonSerializer.Deserialize<T>(bufferWriter.WrittenSpan, options);
    }

    public void OnMessage(IsblNet net, JsonElement node)
    {
        if (!node.TryGetProperty("action", out var actionElement)) throw new System.Exception("Missing property action");
        var action = actionElement.GetString();
        if (string.IsNullOrEmpty(action)) throw new System.Exception("Received invalid action");
        if (action == "begin")
        {
            var leader = node.GetProperty("leader").GetUInt16();
            if (leader == net.SelfId && node.GetProperty("asLeader").GetBoolean())
            {
                Utils.Log("Starting local calibration");
                _calibrationsLocal.Add(new LocalCalibration()
                {
                    FollowerDeviceId = node.GetProperty("followerDevice").GetUInt16(),
                    LocalDeviceId = node.GetProperty("leaderDevice").GetUInt16(),
                    FollowerId = node.GetProperty("follower").GetUInt16(),
                    StartTimeStamp = IsblStaticXRDevice.GetTimeNow(),
                });
            }
            else
            {
                Utils.Log("Starting remote calibration");
                _calibrationsRemote.Add(new RemoteCalibration()
                {
                    LocalDeviceId = node.GetProperty("followerDevice").GetUInt16(),
                    LeaderId = leader,
                    StartTimeStamp = IsblStaticXRDevice.GetTimeNow(),
                    SampleBatch = new(),
                });
            }
        }
        else if (action == "samples")
        {
            var followerDeviceId = node.GetProperty("deviceId").GetUInt16();
            var followerId = node.GetProperty("clientId").GetUInt16();
            var cal = _calibrationsLocal.Find(c => c.FollowerId == followerId && c.FollowerDeviceId == followerDeviceId);
            if (cal == null) throw new Exception("Received samples for unknown calibration");
            if (cal.FinishedTimeStamp < 1)
            {
                var samples = JsonElementToObject<Sample[]>(node.GetProperty("samples"));
                Utils.Log($"Received {samples.Length} sample{(samples.Length > 1 ? "s" : "")}");
                cal.FollowerSamples.AddRange(samples);
            }
        }
        else if (action == "end")
        {
            var followerDevice = node.GetProperty("followerDevice").GetUInt16();
            var follower = node.GetProperty("follower").GetUInt16();
            var leader = node.GetProperty("leader").GetUInt16();
            var leaderDevice = node.GetProperty("leaderDevice").GetUInt16();

            string logInfo = $"leader: {leader}\nleaderDevice: {leaderDevice}\nfollower: {follower}\nfollowerDevice: {followerDevice}";
            if (leader == net.SelfId)
            {
                var indexLocal = _calibrationsLocal.FindIndex(c => c.FollowerDeviceId == followerDevice && c.FollowerId == follower && c.LocalDeviceId == leaderDevice);
                if (indexLocal >= 0)
                {
                    var startTimeStamp = _calibrationsLocal[indexLocal].StartTimeStamp;
                    _calibrationsLocal.RemoveAt(indexLocal);
                    Utils.Log($"Removed local calibration.\n{logInfo}\nStartTimeStamp:{startTimeStamp}");
                }
                else
                {
                    Utils.Log($"Tried to remove local calibration, but none was found. Did it finish already?\n{logInfo}");
                }
            }
            if (follower == net.SelfId)
            {
                var indexRemote = _calibrationsRemote.FindIndex(c => c.LeaderId == leader && c.LocalDeviceId == followerDevice);
                if (indexRemote >= 0)
                {
                    var startTimeStamp = _calibrationsRemote[indexRemote].StartTimeStamp;
                    _calibrationsRemote.RemoveAt(indexRemote);
                    Utils.Log($"Removed remote calibration.\n{logInfo}\nStartTimeStamp:{startTimeStamp}");
                }
                else
                {
                    Utils.Log($"Tried to remove remote calibration, but none was found. Did it timeout already?\n{logInfo}");
                }
            }
        }
    }

    public void Reset()
    {
        _calibrationsLocal = new();
        _calibrationsRemote = new();
    }

    bool TryCollectLocalSample(IsblNet net, ushort deviceId, double startTimeStamp, out Sample sample)
    {
        var dev = net.DeviceManager.Devices.Find(d => d.LocalDevice.LocallyUniqueId == deviceId);
        if (!dev)
        {
            sample = new();
            return false;
        }
        sample = new Sample()
        {
            Timestamp = dev.NetDevice.LastUpdateTime - startTimeStamp,
            Position = dev.NetDevice.DevicePosition,
            Rotation = dev.NetDevice.DeviceRotation,
        };
        return true;
    }

    private bool _redoneCalibration = false;

    public void Tick(IsblNet net)
    {
        var now = IsblStaticXRDevice.GetTimeNow();
        _calibrationsRemote.RemoveAll(c =>
        {
            var res = c.StartTimeStamp + Timeout < now;
            if (res) Utils.Log($"Remote calibration timeout. leader: {c.LeaderId}, localDevice: {c.LocalDeviceId}, start + timeout < now: {c.StartTimeStamp} + {Timeout} < {now}");
            return res;
        });

        _calibrationsLocal.RemoveAll(c =>
        {
            var res = c.StartTimeStamp + Timeout < now && c.FinishedTimeStamp < 1;
            if (res) Utils.Log($"Local calibration timeout.\nfollower: {c.FollowerId}, followerDevice: {c.FollowerDeviceId}, localDevice: {c.LocalDeviceId}\nstart + timeout < now: {c.StartTimeStamp} + {Timeout} < {now}");
            return res;
        });

        _calibrationsLocal.RemoveAll(c =>
        {
            var res = c.FinishedTimeStamp > 1 && c.FinishedTimeStamp + FinishedDeleteTimeout < now;
            if (res) Utils.Log($"Clearing out finished local calibration. follower: {c.FollowerId}, followerDevice: {c.FollowerDeviceId}, localDevice: {c.LocalDeviceId}");
            return res;
        });

        foreach (var c in _calibrationsRemote)
        {
            if (TryCollectLocalSample(net, c.LocalDeviceId, c.StartTimeStamp, out var sample))
            {
                c.SampleBatch.Add(sample);
                if (c.SampleBatch.Count >= SampleBatchSize)
                {
                    Utils.Log($"Sending {SampleBatchSize} samples: eg. {sample.Position} {sample.Rotation}");
                    net.Socket.SendAsync(new
                    {
                        feature = FeatureId,
                        action = "samples",
                        deviceId = c.LocalDeviceId,
                        clientId = net.SelfId,
                        leader = c.LeaderId,
                        samples = c.SampleBatch.ToArray(),
                    });
                    c.SampleBatch.Clear();
                }
            }
            else
            {
                // Disconnection (missing device) is handled by timeout.
                Utils.Log($"Failed to collect sample to send. DeviceId: {c.LocalDeviceId}. Devices: {string.Join(", ", net.DeviceManager.Devices.Select(d => d.LocalDevice.LocallyUniqueId))}");
            }

        }

        foreach (var c in _calibrationsLocal)
        {
            if (c.FinishedTimeStamp > 1) continue;

            if (TryCollectLocalSample(net, c.LocalDeviceId, c.StartTimeStamp, out var sample))
            {
                Utils.Log($"Collected local sample: {sample.Position} {sample.Rotation}");
                c.LeaderSamples.Add(sample);
            }
            else
            {
                Utils.Log($"Local sample collect failed. DeviceId: {c.LocalDeviceId}. Devices: {string.Join(", ", net.DeviceManager.Devices.Select(d => d.LocalDevice.LocallyUniqueId))}");
            }
        }

        if (!_redoneCalibration)
        {
            var cal = IsblPersistentData.Instance.LastCalibration;
            Utils.Log($"LastCalibration {cal != null}");
            _redoneCalibration = true;
            if (cal != null)
            {
                Utils.Log("Restoring last calibration...");
                var toRedo = new LocalCalibration { StartTimeStamp = now, FollowerDeviceId = cal.FollowerDeviceId, FollowerId = cal.FollowerId, FollowerSamples = new(), LeaderSamples = new(), LocalDeviceId = cal.LeaderDeviceId };
                foreach (var sample in cal.Samples)
                {
                    toRedo.FollowerSamples.Add(sample.Follower);
                    toRedo.LeaderSamples.Add(sample.Leader);
                }
                ComputeCalibration(toRedo, out var translate, out var rotate);
            }
        }

        static bool IsCalibrationReady(LocalCalibration c) => c.LeaderSamples.Count >= RequiredSamples && c.FollowerSamples.Count >= RequiredSamples && c.FinishedTimeStamp < 1;

        foreach (var cal in _calibrationsLocal)
        {
            if (!IsCalibrationReady(cal)) continue;
            Utils.Log("Calibration ready, computing...");
            IsblPersistentData.Update(data =>
            {
                data.LastCalibration = new IsblPersistentData.Calibration()
                {
                    FollowerDeviceId = cal.FollowerDeviceId,
                    FollowerId = cal.FollowerId,
                    LeaderDeviceId = cal.LocalDeviceId,
                    LeaderId = net.SelfId,
                    Samples = new(),
                };
                for (int i = 0; i < RequiredSamples; ++i)
                {
                    var leaderSample = cal.LeaderSamples[i];
                    var followerSample = cal.FollowerSamples[i];
                    data.LastCalibration.Samples.Add(new IsblPersistentData.CalibrationSample { Leader = leaderSample, Follower = followerSample });
                }
            });

            cal.FinishedTimeStamp = now;
            _ = net.Socket.SendAsync(new
            {
                feature = FeatureId,
                action = "end",
                follower = cal.FollowerId,
                followerDevice = cal.FollowerDeviceId,
                leader = net.SelfId,
                leaderDevice = cal.LocalDeviceId,
            });

            // TODO: Potentially run this in thread if it is too slow?
            ComputeCalibration(cal, out var translate, out var rotate);
            translate = Vector3.zero;
            //Utils.Log("Zeroed out translate");

            net.Socket.SendAsync(new
            {
                action = "multiset",
                data = new object[] {
                    new {
                        field = "calibration",
                        client = net.SelfId,
                        value = new Isbl.NetServerState.Calibration() {
                            Rotate = rotate,
                            Scale = Vector3.one,
                            Translate = translate,
                        }
                    },
                },
            });
        }
    }

    /**
     * Performs Unity to OVR data conversion, runs the calculation and converts back
     */
    static void ComputeCalibration(LocalCalibration cal, out Vector3 translate, out Quaternion rotate)
    {
        IsblCalibration calculation = new();
        for (int i = 0; i < RequiredSamples; ++i)
        {
            var leaderSample = cal.LeaderSamples[i];
            var followerSample = cal.FollowerSamples[i];
            calculation.AddPair(
                UnityToOVR(leaderSample.Position),
                UnityToOVR(leaderSample.Rotation),
                UnityToOVR(followerSample.Position),
                UnityToOVR(followerSample.Rotation)
            );
        }
        var result = calculation.Compute();
        translate = OVRToUnity(new Vector3((float)result.X, (float)result.Y, (float)result.Z));
        //var rotateRad = new Vector3((float)result.Rex, (float)result.Rey, (float)result.Rez);
        rotate = OVRToUnity(new Quaternion((float)result.Rqx, (float)result.Rqy, (float)result.Rqz, (float)result.Rqw));

        //Quaternion rotate = Quaternion.Euler(rotateRad * (180f / MathF.PI));

        Utils.Log("Calibration result:"
        + $"\n  Translate: ({result.X}, {result.Y}, {result.Z})"
        + $"\n  Rotate euler: ({result.Rex}, {result.Rey}, {result.Rez})"
        + $"\n  Rotate quaternion: ({result.Rqx}, {result.Rqy}, {result.Rqz}, {result.Rqw})"
        + "\nConverted:"
        + $"\n  Translate: ({translate.x}, {translate.y}, {translate.z})"
        + $"\n  Rotate euler: ({rotate.eulerAngles.x}, {rotate.eulerAngles.y}, {rotate.eulerAngles.z})"
        + $"\n  Rotate quaternion: ({rotate.x}, {rotate.y}, {rotate.z}, {rotate.w})"
        );
    }

    /**
     * Converts Vector3 from OVR's right-handed coordinate system to Unity's
     * left-handed coordinates.
     */
    static Vector3 OVRToUnity(Vector3 rightHandedVector)
    {
        return new Vector3(rightHandedVector.x, rightHandedVector.y, -rightHandedVector.z);
    }

    /**
     * Converts Vector3 from Unity's left-handed coordinate system to OVR's
     * right-handed coordinates.
     */
    static Vector3 UnityToOVR(Vector3 leftHandedVector)
    {
        return new Vector3(leftHandedVector.x, leftHandedVector.y, -leftHandedVector.z);
    }

    /**
     * Converts Quaternion from OVR's right-handed coordinate system to Unity's
     * left-handed coordinates.
     */
    static Quaternion OVRToUnity(Quaternion rightHandedQuaternion)
    {
        return new Quaternion(-rightHandedQuaternion.x,
                              -rightHandedQuaternion.y,
                              rightHandedQuaternion.z,
                              rightHandedQuaternion.w);
    }

    /**
     * Converts Quaternion from Unity's left-handed coordinate system to OVR's
     * right-handed coordinates.
     */
    static Quaternion UnityToOVR(Quaternion leftHandedQuaternion)
    {
        return new Quaternion(-leftHandedQuaternion.x,
                              -leftHandedQuaternion.y,
                              leftHandedQuaternion.z,
                              leftHandedQuaternion.w);
    }
}
