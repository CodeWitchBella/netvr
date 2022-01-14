# netvr-unity

This folder contains source files for unity part netvr project. To build this you need unity.

To install unity I recommend using [Unity Hub](https://unity3d.com/get-unity/download). Install it and in projects tab click on arrow next to open and choose _Add project from disk_, in a window that appears select `netvr-unity` folder. This will show you the required editor version. Open [Unity download archive](https://unity3d.com/get-unity/download/archive) and there click on Unity Hub button next to correct version. This will again open in Unity Hub, select _Android Build Support_ along with _Android SDK & NDK Tools_ and _OpenJDK_.

You will also need netvr-cpp .dll and/or .so files to run this. You can follow directions in netvr-cpp folder to get that. You can also download prebuilt binaries from CI (instructions on how to do that are also in netvr-cpp).

Once all that is done you can just click on the project in project list to open it.

To build the app you can use `Build` entry in the top menu.
