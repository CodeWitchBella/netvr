using UnityEngine;
using UnityEngine.InputSystem.XR;
using GLTFast;
using System.Threading.Tasks;

public class XRTrackedController : MonoBehaviour
{
    public bool LeftHand = true;

    string _initializedControllerName = "";

    async Task<GltfImport> LoadModel(TrackedObjectModel info, string controllerName)
    {
        if (info == null)
        {
            Debug.LogError(string.Format("Unknown controller \"{0}\"", controllerName));
            return null;
        }

        var gltf = new GltfImport();
        var success = await gltf.Load(info.ModelPath);

        if (!success)
        {
            Debug.LogError("Loading glTF failed!");
            return null;
        }
        return gltf;
    }

    async void InitializeIfNeeded(XRController controller)
    {
        var currentName = controller?.name ?? "none";
        if (currentName == _initializedControllerName) return;
        _initializedControllerName = currentName;

        var builder = TrackedObjectModel.GetInfo(controllerName: controller.name, leftHand: LeftHand);
        var gltf = await LoadModel(builder, controller.name);

        // cleanup after we loaded model, but also if no model was found
        for (var i = 0; i < transform.childCount; ++i)
            Destroy(transform.GetChild(i).gameObject);

        if (gltf != null)
        {
            gltf.InstantiateMainScene(transform);
            var model = transform.GetChild(0);
            var parent = model.parent;
            var root = model.GetChild(0);
            root.parent = parent;
            Destroy(model.gameObject);
            root.localPosition = builder.Position;
            root.localEulerAngles = builder.Rotation;
        }
    }

    void Update()
    {
        var controller = LeftHand ? XRController.leftHand : XRController.rightHand;
        InitializeIfNeeded(controller);
        if (controller == null) return;

        var pos = controller.devicePosition.ReadValue();
        var rot = controller.deviceRotation.ReadValue();

        gameObject.transform.localPosition = pos;
        gameObject.transform.localRotation = rot;
    }
}
