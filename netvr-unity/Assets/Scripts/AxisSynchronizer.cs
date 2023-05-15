using UnityEngine;

public class AxisSynchronizer : MonoBehaviour
{
    public System.Func<float> ValueGetter;
    public System.Func<bool> VisibilityGetter;
    Transform _min;
    Transform _max;
    MeshRenderer _meshRenderer;

    void Start()
    {
        _min = transform.parent.Find(gameObject.name.Replace("_value", "_min"));
        _max = transform.parent.Find(gameObject.name.Replace("_value", "_max"));
        _meshRenderer = transform.GetComponentInChildren<MeshRenderer>();
    }

    void Update()
    {
        if (ValueGetter != null && _min != null && _max != null)
        {
            var value = ValueGetter();
            transform.localPosition = Vector3.Lerp(_min.localPosition, _max.localPosition, value);
            transform.localRotation = Quaternion.Slerp(_min.localRotation, _max.localRotation, value);
        }
        if (VisibilityGetter != null && _meshRenderer != null)
        {
            _meshRenderer.enabled = VisibilityGetter();
        }
    }
}
