using System.Collections;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using UnityEngine;

public class IsblNetComponent : MonoBehaviour
{
    static IsblNetComponent _instance;
    public static IsblNet Instance
    {
        get
        {
            if (_instance != null) return _instance._net;
            _instance = FindObjectOfType<IsblNetComponent>();
            if (_instance != null) return _instance._net;
            GameObject go = new("IsblNet");
            _instance = go.AddComponent<IsblNetComponent>();
            if (_instance._net == null) _instance._net = new();
            return _instance._net;
        }
    }
    public static bool InstanceExists { get { return _instance != null; } }
    void Start()
    {
        if (_instance == null)
        {
            _instance = this;
        }
        else if (_instance != this)
        {
            Destroy(gameObject);
            Debug.LogWarning("Only one instance of IsblNet could be in same scene");
            return;
        }

        DontDestroyOnLoad(gameObject); // only explicit destroying
        if (_net == null)
        {
            _net = new IsblNet();
        }
    }

    void OnDestroy()
    {
        _net.Dispose();
        _net = null;
    }

#if UNITY_EDITOR
    public class SelfPropertyAttribute : PropertyAttribute { };
    [SelfProperty]
    public GameObject Hack;
#endif
    IsblNet _net;
}
