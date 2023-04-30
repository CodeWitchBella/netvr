using System.Collections.Generic;
using UnityEngine;
using System.Linq;
using System;
using UnityEngine.XR.OpenXR;


namespace Isbl.NetVR
{

    public class IsblRemoteDevice : MonoBehaviour
    {
        public UInt32 Id { get; internal set; }
        public string InteractionProfile { get; internal set; }
        public string SubactionPath { get; internal set; }

#if UNITY_EDITOR
    public class SelfPropertyAttribute : PropertyAttribute { };
    [SerializeField]
    [SelfProperty]
    public IsblRemoteDevice EditorOnly;
#endif
        void OnEnable()
        {
#if UNITY_EDITOR
            EditorOnly = this;
#endif
        }

        void OnDisable()
        {
#if UNITY_EDITOR
            EditorOnly = null;
#endif
        }
    }

}
