using System.Collections;
using System.Collections.Generic;
using UnityEngine;

[RequireComponent(typeof(ButtonResponder))]
public class ToggleLog : MonoBehaviour
{
    public TMPro.TextMeshPro TextMesh;
    string _template;

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
        if (name == "ToggleLog")
        {
            IsblPersistentData.Instance.LogLocalData = !IsblPersistentData.Instance.LogLocalData;
            ReText();
        }
    }

    void Update()
    {
        ReText();
    }

    void ReText()
    {
        var data = IsblPersistentData.Instance;
        TextMesh.text = _template
            .Replace("{log}", data.LogLocalData ? "yes" : "no");
    }
}
