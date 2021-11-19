using System.Collections;
using System.Collections.Generic;
using UnityEngine;

[RequireComponent(typeof(TMPro.TextMeshPro))]
public class SocketUrl : MonoBehaviour
{
    TMPro.TextMeshPro _textMeshPro;
    string _template;
    void Start()
    {
        _textMeshPro = GetComponent<TMPro.TextMeshPro>();
        _template = _textMeshPro.text;
        if (string.IsNullOrEmpty(_template))
        {
            _template = "{SocketUrl}";
        }
    }

    void Update()
    {
        _textMeshPro.text = _template.Replace("{SocketUrl}", IsblNet.Instance.SocketUrl);
    }
}
