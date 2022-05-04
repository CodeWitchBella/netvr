using System.Collections;
using System.Collections.Generic;
using UnityEngine;

[RequireComponent(typeof(ButtonResponder))]
public class ServerSelector : MonoBehaviour
{
    public TMPro.TextMeshPro TextMesh;
    string _template;
    int _nextId = 0;

    void Start()
    {
        _template = TextMesh.text;
        ReText();
    }

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
        Utils.Log($"ProximityButtonClicked {name}");
        if (name == "Next")
        {
            _nextId++;
            _nextId %= IsblConfig.Instance.GetConnectionCount();
            ReText();
        }
        else if (name == "Switch")
        {
            IsblNet.Instance.SocketUrl = IsblConfig.Instance.GetConnectionSocketUrl(_nextId);
            _nextId = 0;
            ReText();
        }
    }

    void Update()
    {
        ReText();
    }

    void ReText()
    {
        var data = IsblConfig.Instance;
        TextMesh.text = _template
            .Replace("{current}", IsblNet.Instance.SocketUrl)
            .Replace("{next}", data.GetConnectionSocketUrl(_nextId));
    }
}
