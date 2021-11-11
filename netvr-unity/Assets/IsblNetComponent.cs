using System.Collections;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using UnityEngine;

/// <summary>
/// This Unity Component makes sure that there is only a single IsblNet instance
/// in existence at the same time. There is no reason to access this directly
/// from code. Use IsblNet.Instance instead.
/// </summary>
///
/// It also provides GUI with a few extra bits of information and makes sure
/// that IsblNet gets properly disposed.
public class IsblNetComponent : MonoBehaviour
{
    static IsblNetComponent _instance;
    /// <summary>
    /// Unity part of IsblNet singleton scheme.
    /// </summary>
    public static IsblNet Instance
    {
        get { return _instance._net; }
        set
        {
            if (_instance == null) _instance = FindObjectOfType<IsblNetComponent>();

            if (_instance == null)
            {
                GameObject go = new("IsblNet");
                _instance = go.AddComponent<IsblNetComponent>();
            }

            if (_instance._net != value)
            {
                _instance._net?.Dispose();
            }
            _instance._net = value;
        }
    }
    public static bool InstanceExists { get { return _instance?._net != null; } }
    void Start()
    {
        if (_instance == null)
        {
            _instance = this;
            IsblNet.EnsureInstanceExists();
        }
        else if (_instance != this)
        {
            Destroy(gameObject);
            Debug.LogWarning("Only one instance of IsblNet could be in same scene");
            return;
        }

        DontDestroyOnLoad(gameObject); // only explicit destroying
    }

    void FixedUpdate()
    {
        _net?.Tick();
    }

    void OnDestroy()
    {
        _net?.Dispose();
        _net = null;
    }

#if UNITY_EDITOR
    public class SelfPropertyAttribute : PropertyAttribute { };
    [SelfProperty]
    public GameObject Hack;
#endif
    IsblNet _net;
}
