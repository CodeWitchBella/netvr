using UnityEngine;

public class AxisSynchronizer : MonoBehaviour
{
    public System.Func<float> ValueGetter;
    public System.Func<bool> VisibilityGetter;
    public Transform Min;
    public Transform Max;
    public float Value;
    public MeshRenderer MeshRenderer;

    void Start()
    {
        Min = transform.parent.Find(gameObject.name.Replace("_value", "_min"));
        Max = transform.parent.Find(gameObject.name.Replace("_value", "_max"));
        MeshRenderer = FindObjectOfType<MeshRenderer>();
    }

    void Update()
    {
        if (ValueGetter != null)
        {
            Value = ValueGetter();
            transform.localPosition = Vector3.Lerp(Min.localPosition, Max.localPosition, Value);
            transform.localRotation = Quaternion.Slerp(Min.localRotation, Max.localRotation, Value);
        }
        if (VisibilityGetter != null && MeshRenderer != null)
        {
            MeshRenderer.enabled = VisibilityGetter();
        }
    }
}
