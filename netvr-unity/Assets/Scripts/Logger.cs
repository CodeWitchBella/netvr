using System.Collections;
using System.Collections.Generic;
using UnityEngine;

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
            string path = $"{dir}/{System.DateTime.Now.ToString("yyyy-MM-dd_HH-mm-ss")}.txt";
            System.IO.File.WriteAllText(path, log);
            Debug.Log("Wrote to " + path);
            log = "";
        }
        logging = !logging;
    }

    void Log()
    {
        log += Time.time;

        foreach (var device in local.Devices)
        {
            log += "\tlocal";
            log += "\t" + device.NetDevice.LocallyUniqueId;
            log += "\t" + device.NetDevice.DevicePosition;
            log += "\t" + device.NetDevice.DeviceRotation;
            log += "\t" + device.NetDevice.DeviceRotation.eulerAngles;
        }
        foreach (var device in remote.Devices)
        {
            log += "\tremote";
            log += "\t" + device.Id;
            log += "\t" + device.InteractionProfile;
            log += "\t" + device.SubactionPath;
            log += "\t" + device.transform.position;
            log += "\t" + device.transform.rotation;
            log += "\t" + device.transform.rotation.eulerAngles;
        }
        log += "\n";
    }
}
