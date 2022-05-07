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
    const double MinSamplingDelay = 0.05;
    const int RequiredSamples = 1000; // OVR SpaceCal equivalents: FAST=200, SLOW=500, VERY_SLOW=1000
    const int SampleBatchSize = 10;

    class FollowerCalibration
    {
        public ushort LeaderId, LocalDeviceId;
        public double StartTimeStamp;
        public double LastSampleTargetTimeStamp;
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

    class LeaderCalibration
    {
        [JsonInclude]
        [JsonPropertyName("followerDevice")]
        public ushort FollowerDeviceId;
        [JsonInclude]
        [JsonPropertyName("leaderDevice")]
        public ushort LeaderDeviceId;
        [JsonInclude]
        [JsonPropertyName("follower")]
        public ushort FollowerId;
        [JsonInclude]
        [JsonPropertyName("leader")]
        public ushort LeaderId;
        [JsonInclude]
        [JsonPropertyName("startTimeStamp")]
        public double StartTimeStamp;
        [JsonInclude]
        [JsonPropertyName("finishedTimeStamp")]
        public double FinishedTimeStamp = 0;

        public double LastSampleTargetTimeStamp;

        [JsonConverter(typeof(Isbl.Json.QuaternionAsEulerConverter))]
        [JsonInclude]
        [JsonPropertyName("resultRotate")]
        public Quaternion ResultRotate;

        [JsonConverter(typeof(Isbl.Json.Vector3Converter))]
        [JsonInclude]
        [JsonPropertyName("resultTranslate")]
        public Vector3 ResultTranslate;

        [JsonInclude]
        [JsonPropertyName("leaderSamples")]
        public List<Sample> LeaderSamples = new();
        [JsonInclude]
        [JsonPropertyName("followerSamples")]
        public List<Sample> FollowerSamples = new();
    }

    List<LeaderCalibration> _calibrationsLocal;
    List<FollowerCalibration> _calibrationsRemote;

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
                _calibrationsLocal.Add(new LeaderCalibration()
                {
                    FollowerDeviceId = node.GetProperty("followerDevice").GetUInt16(),
                    LeaderDeviceId = node.GetProperty("leaderDevice").GetUInt16(),
                    FollowerId = node.GetProperty("follower").GetUInt16(),
                    LeaderId = leader,
                    StartTimeStamp = IsblStaticXRDevice.GetTimeNow(),
                });
            }
            else
            {
                Utils.Log("Starting remote calibration");
                _calibrationsRemote.Add(new FollowerCalibration()
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
                var indexLocal = _calibrationsLocal.FindIndex(c => c.FollowerDeviceId == followerDevice && c.FollowerId == follower && c.LeaderDeviceId == leaderDevice);
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

    bool TryCollectLocalSample(IsblNet net, ushort deviceId, double startTimeStamp, double lastSampleTimeStamp, out Sample sample)
    {
        var dev = net.DeviceManager.Devices.Find(d => d.LocalDevice.LocallyUniqueId == deviceId);
        if (!dev)
        {
            sample = new();
            Utils.Log($"Failed to collect sample. DeviceId: {deviceId}. Devices: {string.Join(", ", net.DeviceManager.Devices.Select(d => d.LocalDevice.LocallyUniqueId))}");
            return false;
        }
        var timestamp = dev.NetDevice.LastUpdateTime - startTimeStamp;
        if (timestamp - lastSampleTimeStamp < MinSamplingDelay)
        {
            sample = new();
            return false;
        }

        sample = new Sample()
        {
            Timestamp = timestamp,
            Position = dev.NetDevice.DevicePosition,
            Rotation = dev.NetDevice.DeviceRotation,
        };
        return true;
    }

    private bool _redoneCalibration = false;

    public void Update(IsblNet net)
    {
        if (_calibrationsRemote != null)
        {
            foreach (var c in _calibrationsRemote)
            {
                if (TryCollectLocalSample(net, c.LocalDeviceId, c.StartTimeStamp, c.LastSampleTargetTimeStamp, out var sample))
                {
                    c.LastSampleTargetTimeStamp += MinSamplingDelay;
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
            }
        }

        if (_calibrationsLocal != null)
        {
            foreach (var c in _calibrationsLocal)
            {
                if (c.FinishedTimeStamp > 1) continue;

                if (TryCollectLocalSample(net, c.LeaderDeviceId, c.StartTimeStamp, c.LastSampleTargetTimeStamp, out var sample))
                {
                    c.LastSampleTargetTimeStamp += MinSamplingDelay;
                    Utils.Log($"Collected local sample: {sample.Position} {sample.Rotation}");
                    c.LeaderSamples.Add(sample);
                }
            }
        }
    }

    public void FixedUpdate(IsblNet net)
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
            if (res) Utils.Log($"Local calibration timeout.\nfollower: {c.FollowerId}, followerDevice: {c.FollowerDeviceId}, localDevice: {c.LeaderDeviceId}\nstart + timeout < now: {c.StartTimeStamp} + {Timeout} < {now}");
            return res;
        });

        _calibrationsLocal.RemoveAll(c =>
        {
            var res = c.FinishedTimeStamp > 1 && c.FinishedTimeStamp + FinishedDeleteTimeout < now;
            if (res) Utils.Log($"Clearing out finished local calibration. follower: {c.FollowerId}, followerDevice: {c.FollowerDeviceId}, localDevice: {c.LeaderDeviceId}");
            return res;
        });

        if (!_redoneCalibration)
        {
            var calFileName = IsblConfig.Instance.LastCalibration;
            _redoneCalibration = true;
            if (!string.IsNullOrEmpty(calFileName)) _ = ReadAndRecompute(calFileName);
        }

        static bool IsCalibrationReady(LeaderCalibration c) => c.LeaderSamples.Count >= RequiredSamples && c.FollowerSamples.Count >= RequiredSamples && c.FinishedTimeStamp < 1;

        foreach (var cal in _calibrationsLocal)
        {
            if (!IsCalibrationReady(cal)) continue;
            Utils.Log("Calibration ready, computing...");

            cal.FinishedTimeStamp = now;
            _ = net.Socket.SendAsync(new
            {
                feature = FeatureId,
                action = "end",
                follower = cal.FollowerId,
                followerDevice = cal.FollowerDeviceId,
                leader = cal.LeaderId,
                leaderDevice = cal.LeaderDeviceId,
            });

            // TODO: Potentially run this in thread if it is too slow?
            ComputeCalibration(cal);
            IsblConfig.Update((data) => data.LastCalibration = SaveCalibration(cal));

            net.Socket.SendAsync(new
            {
                action = "multiset",
                data = new object[] {
                    new {
                        field = "calibration",
                        client = cal.FollowerId,// net.SelfId,
                        value = new Isbl.NetServerState.Calibration() {
                            Rotate = cal.ResultRotate,
                            Scale = Vector3.one,
                            Translate = cal.ResultTranslate,
                        }
                    },
                },
            });
        }
    }

    static async System.Threading.Tasks.Task ReadAndRecompute(string filename)
    {
        var cal = await Isbl.Persistent.DataDirectory.ReadFile<LeaderCalibration>(filename);
        Utils.Log($"Recomputing last calibration with {cal.LeaderSamples.Count} leader and {cal.FollowerSamples.Count} follower samples");

        try { ComputeCalibration(cal); }
        catch (Exception e) { Debug.LogException(e); }
        // SaveCalibration(cal, filename);
    }

    static string SaveCalibration(LeaderCalibration cal, string logFile = null)
    {
        if (logFile == null)
            logFile = $"calibration-{$"{DateTime.UtcNow:o}".Replace(":", "-")[..19]}.json";
        Utils.Log($"Saving Calibration {logFile}");
        Isbl.Persistent.DataDirectory.WriteFile(logFile, cal);
        return logFile;
    }

    /**
     * Performs Unity to OVR data conversion, runs the calculation and converts back
     */
    static void ComputeCalibration(LeaderCalibration cal)
    {
        using var timer = new IsblStopwatch("ComputeCalibration");
        IsblCalibration calculation = new();
        var sampleCount = Math.Min(cal.LeaderSamples.Count, cal.FollowerSamples.Count);
        for (int i = 0; i < sampleCount; ++i)
        {
            var leaderSample = cal.LeaderSamples[i];
            var followerSample = cal.FollowerSamples[i];
            var p1 = UnityToOVR(leaderSample.Position);
            var q1 = UnityToOVR(leaderSample.Rotation);
            var p2 = UnityToOVR(followerSample.Position);
            var q2 = UnityToOVR(followerSample.Rotation);
            calculation.AddPair(p1, q1, p2, q2);
            if (false)
            {
                Utils.Log($"Passing converted sample {i}:"
                    + $"\np1: ({p1.x}, {p1.y}, {p1.z})"
                    + $"\nq1: ({q1.x}, {q1.y}, {q1.z}, {q1.w})"
                    + $"\np2: ({p2.x}, {p2.y}, {p2.z})"
                    + $"\nq2: ({q2.x}, {q2.y}, {q2.z}, {q2.w})"
                );
            }
        }
        IsblDynamicLibrary.CalibrationComputeResult result;
        using (var timer2 = new IsblStopwatch("ComputeCalibration::compute only"))
        {
            result = calculation.Compute();
        }
        cal.ResultTranslate = OVRToUnity((float)result.X, (float)result.Y, (float)result.Z);
        //var rotateRad = new Vector3((float)result.Rex, (float)result.Rey, (float)result.Rez);
        cal.ResultRotate = OVRToUnity((float)result.Rqx, (float)result.Rqy, (float)result.Rqz, (float)result.Rqw);
        //Utils.Log($"Hacking rotate conversion");
        //cal.ResultRotate = Quaternion.Euler(0, cal.ResultRotate.eulerAngles.y, 0);

        //Quaternion rotate = Quaternion.Euler(rotateRad * (180f / MathF.PI));

        Utils.Log("Calibration result:"
        + $"\n  Translate: ({result.X}, {result.Y}, {result.Z})"
        + $"\n  Rotate euler radians: ({result.Rex}, {result.Rey}, {result.Rez})"
        + $"\n  Rotate euler degrees: ({result.Rex / Math.PI * 180d}, {result.Rey / Math.PI * 180d}, {result.Rez / Math.PI * 180d})"
        + $"\n  Rotate quaternion: ({result.Rqx}, {result.Rqy}, {result.Rqz}, {result.Rqw})"
        + "\nConverted:"
        + $"\n  Translate: ({cal.ResultTranslate.x}, {cal.ResultTranslate.y}, {cal.ResultTranslate.z})"
        + $"\n  Rotate euler degrees: ({cal.ResultRotate.eulerAngles.x}, {cal.ResultRotate.eulerAngles.y}, {cal.ResultRotate.eulerAngles.z})"
        + $"\n  Rotate quaternion: ({cal.ResultRotate.x}, {cal.ResultRotate.y}, {cal.ResultRotate.z}, {cal.ResultRotate.w})"
        );
    }

    /**
     * Converts Vector3 from OVR's right-handed coordinate system to Unity's
     * left-handed coordinates.
     */
    static Vector3 OVRToUnity(float x, float y, float z) => new(x, y, -z);

    /**
     * Converts Vector3 from Unity's left-handed coordinate system to OVR's
     * right-handed coordinates.
     */
    static Vector3 UnityToOVR(Vector3 leftHandedVector)
    {
        return new(leftHandedVector.x, leftHandedVector.y, -leftHandedVector.z);
    }

    /**
     * Converts Quaternion from OVR's right-handed coordinate system to Unity's
     * left-handed coordinates.
     */
    static Quaternion OVRToUnity(float x, float y, float z, float w) => new(-x, -y, z, w);

    /**
     * Converts Quaternion from Unity's left-handed coordinate system to OVR's
     * right-handed coordinates.
     */
    static Quaternion UnityToOVR(Quaternion leftHandedQuaternion)
    {
        return new(-leftHandedQuaternion.x,
                   -leftHandedQuaternion.y,
                   leftHandedQuaternion.z,
                   leftHandedQuaternion.w);
    }
}
