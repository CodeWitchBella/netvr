using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Networking;

[DefaultExecutionOrder(1)]
public class Logger : MonoBehaviour
{
    string log = "";
    bool logging = false;
    bool wasPressed = false;

    IsblLocalXRDeviceManager local;
    Isbl.NetVR.IsblRemoteDeviceManager remote;

    void Start()
    {
        local = FindObjectOfType<IsblLocalXRDeviceManager>();
        remote = FindObjectOfType<Isbl.NetVR.IsblRemoteDeviceManager>();
    }

    void Update()
    {
        if (logging) Log();

        bool isPressed = local.Devices.Exists(d => d.NetDevice.PrimaryButton || d.NetDevice.SecondaryButton);
        if (isPressed && !wasPressed) OnPress();
        wasPressed = isPressed;
    }

    private static string GetLogDirectory()
    {
        if (Application.platform == RuntimePlatform.Android) return System.IO.Path.Combine(Application.persistentDataPath, "log");
        return System.IO.Path.Combine(System.IO.Directory.GetCurrentDirectory(), "log");
    }

    void OnPress()
    {
        if (logging)
        {
            // create directory if not exists
            string dir = GetLogDirectory();
            if (!System.IO.Directory.Exists(dir)) System.IO.Directory.CreateDirectory(dir);
            // Write to file
            string fname = $"{System.DateTime.Now.ToString("yyyy-MM-dd_HH-mm-ss")}.txt";
            string path = $"{dir}/{fname}";
            System.IO.File.WriteAllText(path, log);
            Debug.Log("Wrote to " + path);
            StartCoroutine(Upload(log, fname));
            log = "";
        }
        logging = !logging;
    }

    IEnumerator Upload(string data, string fname)
    {
        var feature = Isbl.NetVR.IsblXRFeature.Instance;
        var serverAddress = feature.GetServerAddress().Split(':')[0];
        var path = $"http://{serverAddress}:13161/upload/{fname}";
        Debug.Log($"Uploading to {path}");
        UnityWebRequest www = UnityWebRequest.Put(path, log);
        yield return www.SendWebRequest();

        if (www.result != UnityWebRequest.Result.Success) Debug.Log(www.error);
        else Debug.Log("Upload complete!");
    }

    void Log()
    {
        log += Time.time;

        foreach (var device in local.Devices)
        {
            log += "\tlocal";
            log += "\t" + device.NetDevice.LocallyUniqueId;
            log += "\t" + System.String.Join(",", device.NetDevice.SerializeCharacteristics());
            log += "\t" + Serialize(device.NetDevice.DevicePosition);
            log += "\t" + Serialize(device.NetDevice.DeviceRotation);
            log += "\t" + Serialize(device.NetDevice.DeviceRotation.eulerAngles);
        }
        foreach (var device in remote.Devices)
        {
            log += "\tremote";
            log += "\t" + device.Id;
            log += "\t" + device.InteractionProfile;
            log += "\t" + device.SubactionPath;
            log += "\t" + Serialize(device.transform.position);
            log += "\t" + Serialize(device.transform.rotation);
            log += "\t" + Serialize(device.transform.rotation.eulerAngles);
        }
        log += "\n";
    }

    private string Serialize(float v)
    {
        return v.ToString("R").Replace(',', '.');
    }
    private string Serialize(Vector3 v)
    {
        return $"({Serialize(v.x)}, {Serialize(v.y)}, {Serialize(v.z)})";
    }
    private string Serialize(Quaternion v)
    {
        return $"({Serialize(v.x)}, {Serialize(v.y)}, {Serialize(v.z)}, {Serialize(v.w)})";
    }
}
