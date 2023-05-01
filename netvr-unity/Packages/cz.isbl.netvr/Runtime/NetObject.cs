using System.Collections.Generic;
using UnityEngine;
using System.Linq;
using System;
using UnityEngine.XR.OpenXR;


namespace Isbl.NetVR
{

    public class NetObject : MonoBehaviour
    {
        internal System.UInt32 Id;

        internal bool JustGrabbed = false;
        internal bool JustReleased = false;
        private GameObject _grabbedBy;
        public GameObject GrabbedBy
        {
            get { return _grabbedBy; }
            set
            {
                if (_grabbedBy == value) return;
                _grabbedBy = value;
                if (value != null) JustGrabbed = true;
                if (value == null) JustReleased = true;
            }
        }

        internal Vector3 RequestedPosition;
        internal Quaternion RequestedRotation;
        public void RequestMove(Vector3 position, Quaternion rotation)
        {
            RequestedPosition = position;
            RequestedRotation = rotation;
        }
    }
}
