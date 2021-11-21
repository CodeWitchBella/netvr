using System;
using UnityEngine;

/// <summary>
/// Acts as a generic bridge between buttons and their parents so that relations
/// do not have to be setup via editor (except for naming buttons).
/// </summary>
public class ButtonResponder : MonoBehaviour
{
    public Action<string> OnClick;
}
