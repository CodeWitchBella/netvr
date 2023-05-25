using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Networking;

[DefaultExecutionOrder(1)]
public class Logger : MonoBehaviour
{
    string log = "";
    bool logging = false;
    bool non_continuous = false;
    bool wasPressed = false;
    bool wasBothPressed = false;

    IsblLocalXRDeviceManager local;
    Isbl.NetVR.IsblRemoteDeviceManager remote;

    void Start()
    {
        local = FindObjectOfType<IsblLocalXRDeviceManager>();
        remote = FindObjectOfType<Isbl.NetVR.IsblRemoteDeviceManager>();
    }

    void Update()
    {
        bool isPressed = local.Devices.Exists(d => d.NetDevice.PrimaryButton || d.NetDevice.SecondaryButton);
        bool bothPressed = local.Devices.Exists(d => d.NetDevice.PrimaryButton && d.NetDevice.SecondaryButton);
        if (non_continuous)
        {
            if (isPressed && !wasPressed) Debug.Log("Non continuous resume");
            if (!isPressed && wasPressed) Debug.Log("Non continuous pause");
            if (isPressed) Log();
            if (bothPressed && !wasBothPressed)
            {
                Debug.Log("Non continuous finish");
                FinishNonContinuous();
            }
        }
        else if (bothPressed)
        {
            Debug.Log("Non continuous start");
            non_continuous = true;
            log = "";
        }
        else if (isPressed && !wasPressed)
        {
            OnPress();
        }
        else if (logging)
        {
            Log();
        }
        wasPressed = isPressed;
        wasBothPressed = bothPressed;
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
            StartCoroutine(WriteAndUpload(log));
            log = "";
        }
        logging = !logging;
    }


    void FinishNonContinuous()
    {
        StartCoroutine(WriteAndUpload(log));
        log = "";
    }

    IEnumerator WriteAndUpload(string data)
    {
        // create directory if not exists
        string dir = GetLogDirectory();
        if (!System.IO.Directory.Exists(dir)) System.IO.Directory.CreateDirectory(dir);
        // Write to file
        string fname = $"{System.DateTime.Now.ToString("yyyy-MM-dd_HH-mm-ss")}.txt";
        string path = $"{dir}/{fname}";
        System.IO.File.WriteAllText(path, log);
        Debug.Log("Wrote to " + path);
        // Upload
        var feature = Isbl.NetVR.IsblXRFeature.Instance;
        var serverAddress = feature.GetServerAddress().Split(':')[0];
        var serverPath = $"http://{serverAddress}:13161/upload/{fname}";
        Debug.Log($"Uploading to {serverPath}");
        UnityWebRequest www = UnityWebRequest.Put(serverPath, log);
        yield return www.SendWebRequest();

        if (www.result != UnityWebRequest.Result.Success) Debug.Log(www.error);
        else
        {
            Debug.Log("Upload complete!");
            Debug.Log(www.downloadHandler.text);
        }
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
