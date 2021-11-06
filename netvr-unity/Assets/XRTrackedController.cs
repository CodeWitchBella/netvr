using System.Collections;
using System.Collections.Generic;
using System.IO;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.XR;
using GLTFast;
using System.Threading.Tasks;

public class XRTrackedController : MonoBehaviour
{
    public bool leftHand = true;

    private string initializedControllerName = "";


    async void InitializeIfNeeded(XRController controller)
    {
        if (controller.name == initializedControllerName) return;
        initializedControllerName = controller.name;
        if (controller.name.StartsWith("HTCViveControllerOpenXR"))
        {
            var url = Path.Combine(Application.streamingAssetsPath, "Controllers", "htc-vive", "none.glb");
            var gltf = new GltfImport();
            var success = await gltf.Load(url);

            if (!success)
            {
                Debug.LogError("Loading glTF failed!");
                return;
            }

            gltf.InstantiateMainScene(transform);
            var scene = transform.GetChild(0);
            var root = scene.Find("htc_vive_none");
            root.parent = transform;
            Destroy(scene.gameObject);
            root.localPosition = new Vector3(0, -0.0083999997f, 0.0544999987f);
            root.localEulerAngles = new Vector3(354.950012f, 180, 0);
        }
    }

    void Update()
    {
        var controller = leftHand ? XRController.leftHand : XRController.rightHand;
        if (controller == null) return;

        InitializeIfNeeded(controller);

        var pos = controller.devicePosition.ReadValue();
        var rot = controller.deviceRotation.ReadValue();

        gameObject.transform.localPosition = pos;
        gameObject.transform.localRotation = rot;
    }
}
