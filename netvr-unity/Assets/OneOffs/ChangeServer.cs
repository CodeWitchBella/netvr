using System.Collections;
using System.Collections.Generic;
using UnityEngine;

[RequireComponent(typeof(IsblTrackedPoseDriver))]
public class ChangeServer : MonoBehaviour
{
    IsblTrackedPoseDriver _driver;
    GameObject _menu;
    bool _lastMenuValue;
    bool _lastSwitchValue;
    // Start is called before the first frame update
    void Start()
    {
        _menu = Instantiate(Resources.Load<GameObject>("url"), transform);
        _menu.SetActive(false);
        _driver = GetComponent<IsblTrackedPoseDriver>();
    }

    // Update is called once per frame
    void Update()
    {
        bool menuValue = _driver.NetDevice.MenuButton;
        if (menuValue && !_lastMenuValue)
        {
            _menu.SetActive(!_menu.activeSelf);
        }
        _lastMenuValue = menuValue;
        bool switchValue = (_driver.NetDevice.PrimaryButton || _driver.NetDevice.Primary2DAxisClick);
        if (_menu.activeSelf && switchValue && !_lastSwitchValue)
        {
            IsblNet.Instance.SocketUrl = IsblNet.Instance.SocketUrl.StartsWith("wss://") ? "ws://192.168.1.31:10000" : "wss://netvr.isbl.workers.dev";
        }
        _lastSwitchValue = switchValue;
    }
}
