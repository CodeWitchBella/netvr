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
 * { normal configuration patch message }
 */

class CalibrationFeature : IIsblNetFeature
{
    public const string FeatureId = "calibration";
    const double Timeout = 180.0;
    const int RequiredSamples = 200; // OVR SpaceCal equivalents: FAST=200, SLOW=500, VERY_SLOW=1000

    struct RemoteCalibrationConfiguration
    {
        public ushort LeaderId, LocalDeviceId;
        public double StartTimeStamp;
    }

    struct Sample
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

        public List<Sample> LeaderSamples = new(), FollowerSamples = new();
    }

    List<LocalCalibration> _calibrationsLocal;
    List<RemoteCalibrationConfiguration> _calibrationsRemote;

    /**
     * Deserializes JsonObject to T. Adapted from https://stackoverflow.com/a/59047063
     */
    public static T JsonElementToObject<T>(JsonElement element, JsonSerializerOptions options = null)
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
            var samples = JsonElementToObject<Sample[]>(node.GetProperty("samples"));
            cal.FollowerSamples.AddRange(samples);
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

    public void Tick(IsblNet net)
    {
        var now = IsblStaticXRDevice.GetTimeNow();
        _calibrationsRemote.RemoveAll(c => c.StartTimeStamp + Timeout > now);

        foreach (var c in _calibrationsRemote)
        {
            if (!TryCollectLocalSample(net, c.LocalDeviceId, c.StartTimeStamp, out var sample))
            {
                net.Socket.SendAsync(new
                {
                    feature = FeatureId,
                    action = "samples",
                    deviceId = c.LocalDeviceId,
                    clientId = net.SelfId,
                    samples = new Sample[] { sample }
                });
            }
            // else: Disconnection (missing device) is handled by timeout.

        }

        foreach (var c in _calibrationsLocal)
        {
            if (TryCollectLocalSample(net, c.LocalDeviceId, c.StartTimeStamp, out var sample))
            {
                c.LeaderSamples.Add(sample);
            }
        }

        static bool ReadyPredicate(LocalCalibration c) => c.LeaderSamples.Count > RequiredSamples && c.FollowerSamples.Count > RequiredSamples;
        var ready = _calibrationsLocal.Where(ReadyPredicate).ToArray();
        _calibrationsLocal.RemoveAll(ReadyPredicate);
        foreach (var cal in ready)
        {
            // Potentially run this in thread if it is too slow
            IsblCalibration calculation = new();
            for (int i = 0; i < RequiredSamples; ++i)
            {
                var leaderSample = cal.LeaderSamples[i];
                var followerSample = cal.FollowerSamples[i];
                calculation.AddPair(leaderSample.Position, leaderSample.Rotation, followerSample.Position, followerSample.Rotation);
            }
            var result = calculation.Compute();
            /*
            net.Socket.SendAsync(new
            {
                action = "patch",
                patches = new object[] {
                    new {
                        op = "replace",
                        path = $"/clients/{net.SelfId}/calibration",
                        value = new Isbl.NetServerState.Calibration() {
                            Rotate = new Quaternion((float)result.Qx, (float)result.Qy, (float)result.Qz, (float)result.Qw),
                            Scale = Vector3.one,
                            Translate = new Vector3((float)result.X, (float)result.Y, (float)result.Z),
                        }
                    },
                },
            });
            */
            net.Socket.SendAsync(new
            {
                action = "multiset",
                data = new object[] {
                    new {
                        field = "calibration",
                        client = net.SelfId,
                        value = new Isbl.NetServerState.Calibration() {
                            Rotate = new Quaternion((float)result.Qx, (float)result.Qy, (float)result.Qz, (float)result.Qw),
                            Scale = Vector3.one,
                            Translate = new Vector3((float)result.X, (float)result.Y, (float)result.Z),
                        }
                    },
                },
            });
        }
    }
}