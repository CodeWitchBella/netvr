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

    struct RemoteCalibrationConfiguration
    {
        public ushort LeaderId, LocalDeviceId;
        public double StartTimeStamp;
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
    List<RemoteCalibrationConfiguration> _calibrationsRemote;

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
            if (leader == net.SelfId)
            {
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
                _calibrationsRemote.Add(new RemoteCalibrationConfiguration()
                {
                    LocalDeviceId = node.GetProperty("followerDevice").GetUInt16(),
                    LeaderId = leader,
                    StartTimeStamp = IsblStaticXRDevice.GetTimeNow(),
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
            else
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
                Utils.Log($"Sending a sample: {sample.Position} {sample.Rotation}");
                net.Socket.SendAsync(new
                {
                    feature = FeatureId,
                    action = "samples",
                    deviceId = c.LocalDeviceId,
                    clientId = net.SelfId,
                    leader = c.LeaderId,
                    samples = new Sample[] { sample }
                });
            }
            else
            {
                Utils.Log($"Failed to collect sample to send. DeviceId: {c.LocalDeviceId}. Devices: {string.Join(", ", net.DeviceManager.Devices.Select(d => d.LocalDevice.LocallyUniqueId))}");
            }
            // else: Disconnection (missing device) is handled by timeout.

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

        static bool IsCalibrationReady(LocalCalibration c) => c.LeaderSamples.Count >= RequiredSamples && c.FollowerSamples.Count >= RequiredSamples && c.FinishedTimeStamp < 1;

        if (!_redoneCalibration)
        {
            var cal = IsblPersistentData.Instance.LastCalibration;
            if (cal == null) { _redoneCalibration = true; }
            else if (net.SelfId != cal.LeaderId) { _redoneCalibration = true; }
            else if (net.ServerState.Clients.ContainsKey(cal.FollowerId))
            {
                _redoneCalibration = true;
                Utils.Log("Restoring last calibration...");
                var result = new LocalCalibration { StartTimeStamp = now, FollowerDeviceId = cal.FollowerDeviceId, FollowerId = cal.FollowerId, FollowerSamples = new(), LeaderSamples = new(), LocalDeviceId = cal.LeaderDeviceId };
                _calibrationsLocal.Add(result);
                foreach (var sample in cal.Samples)
                {
                    result.FollowerSamples.Add(sample.Follower);
                    result.LeaderSamples.Add(sample.Leader);
                }
            }
        }

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
            IsblCalibration calculation = new();
            for (int i = 0; i < RequiredSamples; ++i)
            {
                var leaderSample = cal.LeaderSamples[i];
                var followerSample = cal.FollowerSamples[i];
                calculation.AddPair(leaderSample.Position, leaderSample.Rotation, followerSample.Position, followerSample.Rotation);
            }
            var result = calculation.Compute();
            var translate = new Vector3((float)result.X, (float)result.Y, (float)result.Z) / 100f;
            var rotate = Quaternion.Euler(new Vector3((float)result.Rx, (float)result.Ry, (float)result.Rz) / 180f * Mathf.PI);

            Utils.Log($"Calibration result: ({result.X}, {result.Y}, {result.Z}) ({result.Rx}, {result.Ry}, {result.Rz})");
            Utils.Log($"Calibration result converted: ({translate.x}, {translate.y}, {translate.z}) ({rotate.eulerAngles.x}, {rotate.eulerAngles.y}, {rotate.eulerAngles.z})");

            translate = Vector3.zero;
            Utils.Log("Zeroed out translate");

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
}
