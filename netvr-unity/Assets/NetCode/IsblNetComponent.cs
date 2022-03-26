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
    public bool PrintDebug;

    void OnEnable()
    {
        if (IsblNet.Instance != null)
        {
            gameObject.SetActive(false);
            this.enabled = false;
            Destroy(gameObject);
            return;
        }
        _net = new();
        IsblNet.Instance = _net;

#if UNITY_EDITOR
        if (_net != null) _net.UnityEditorOnlyDebug.PrintDebug = PrintDebug;
#endif
    }

    void OnDisable()
    {
        if (IsblNet.Instance == _net) IsblNet.Instance = null;
        _net?.Dispose();
        _net = null;
    }

    void FixedUpdate()
    {
        _net.Tick();

#if UNITY_EDITOR
        if (_net != null) _net.UnityEditorOnlyDebug.PrintDebug = PrintDebug;
        UnityEditor.EditorUtility.SetDirty(this);
#endif
    }

#if UNITY_EDITOR
    /// <summary>
    /// Used from IsblNetDrawer to attach here without requiring to have properties
    /// </summary>
    public class SelfPropertyAttribute : PropertyAttribute { };
    [SelfProperty]
    public GameObject Hack;
#endif
    private IsblNet _net;
}
