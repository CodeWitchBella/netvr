# Building netvr-cpp

This file contains instructions on how to build .dll/.so files for netvr-cpp.
If you don't want to setup C++ toolchain then you can download automatically
generated final files from [GitHub Actions](#download-from-github-actions).

## Download from Github Actions

To get built files from Github Actions do the following:

- open [the build workflow](https://github.com/CodeWitchBella/netvr/actions/workflows/netvr-cpp.yaml?query=branch%3Amain)
- click on latest successful build (first in the list with green checkmark)
- click on Plugins under Artifacts, Plugins.zip will be downloaded
- replace contents of `netvr-unity/Assets/Plugins` folder in this repository
  with contents of zip you just downloaded

## Windows x64

**Note:** You can also download the file from [GitHub Actions](#download-from-github-actions).

To build netvr-cpp .dll for windows you'll need
[Visual Studio 2022](https://visualstudio.microsoft.com/downloads/) compiler
along with cmake provided by it. I recommend using the Visual Studio installer
application to install this. I would provide more detailed instructions, but
microsoft changes the download instructions constantly.

Older or newer version should work too, but 2022 version is what I tested with.

Open **Developer PowerShell for VS 2022**

```powershell
cd C:\Source\netvr\netvr-cpp
cmake . -G "Visual Studio 17 2022" -A x64 -B Win64
cmake --build Win64 --config Release
```

## Android (for Quest 2)

**Note:** You can also download the file from [GitHub Actions](#download-from-github-actions).

To build netvr-cpp .so for Quest 2 you'll need CMake. You can use the same cmake
installation from Windows build. You'll also need Android NDK, which you can get
by using [Unity Hub](https://store.unity.com/download) and installing Unity
2021.2.3f1 along with Android Build Support module.

Open powershell with CMake available, for example **Developer PowerShell for VS 2022**
and run following commands (you'll have to replace paths with correct ones for
your unity install).


```powershell
$ANDROID_NDK="C:\Stuff\Unity Editors\2021.2.3f1\Editor\Data\PlaybackEngines\AndroidPlayer\NDK"
cd C:\Source\netvr\netvr-cpp
cmake . "-DCMAKE_TOOLCHAIN_FILE=$ANDROID_NDK\build\cmake\android.toolchain.cmake" -DCMAKE_SYSTEM_NAME="Android" "-DANDROID_NDK=$ANDROID_NDK" -DANDROID_PLATFORM=android-29 -DANDROID_ABI="arm64-v8a" -B Android -GNinja
cmake --build Android
```
