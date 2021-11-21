using UnityEngine;

/// <summary>
/// Makes game object be tracked with users head, but not rotate with and also
/// not move up and down.
/// </summary>
///
/// Allows for making sure that game object is always close to player without
/// having to glue it to their head.
public class BeltSpace : MonoBehaviour
{
    void Update()
    {
        var camPos = Camera.main.transform.position;
        transform.position = new Vector3(camPos.x, 0, camPos.z);
    }
}
