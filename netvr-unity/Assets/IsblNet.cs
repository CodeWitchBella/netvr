using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class IsblNet : MonoBehaviour
{
    static IsblNet _instance;
    public static IsblNet Instance
    {
        get
        {
            if (_instance != null) return _instance;
            _instance = FindObjectOfType<IsblNet>();
            if (_instance != null) return _instance;
            GameObject go = new("IsblNet");
            _instance = go.AddComponent<IsblNet>();
            return _instance;
        }
    }

    void OnDestroy()
    {
        if (_instance == this) _instance = null;
    }
}
