using System.Collections;
using System.Collections.Generic;
using UnityEngine;

[RequireComponent(typeof(ButtonResponder))]
public class ServerSelector : MonoBehaviour
{
    void OnEnable()
    {
        GetComponent<ButtonResponder>().OnClick += ProximityButtonClicked;
    }

    void OnDisable()
    {
        GetComponent<ButtonResponder>().OnClick -= ProximityButtonClicked;
    }

    void ProximityButtonClicked(string name)
    {
        Debug.Log($"ProximityButtonClicked {name}");
    }
}
