using UnityEngine;

/// <summary>
/// Use to conjure children when Menu button is pressed
/// </summary>
public class ToggleOnMenu : MonoBehaviour
{
    void Start()
    {
        SetChildrenActive(false);
    }

    bool _menuActive = false;
    bool _wasPressed = false;
    void Update()
    {
        var pressed = false;
        foreach (var dev in IsblTrackedPoseDriver.Devices)
        {
            if (dev.NetDevice.MenuButton)
            {
                pressed = true;
                break;
            }
        }

        if (pressed && !_wasPressed)
        {
            _menuActive = !_menuActive;
            SetChildrenActive(_menuActive);
            if (_menuActive)
            {
                transform.position = Camera.main.transform.position;
                transform.rotation = Camera.main.transform.rotation;
            }
        }
        _wasPressed = pressed;
    }

    void SetChildrenActive(bool value)
    {
        for (var i = 0; i < transform.childCount; ++i) transform.GetChild(i).gameObject.SetActive(value);
    }
}
