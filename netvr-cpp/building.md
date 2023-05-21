# Building netvr-cpp

This file contains instructions on how to build .dll/.so files for netvr-cpp.
In final version of netvr does not use this and this source is included only as
a curiosity of the development process. However, you can still use this code in
form of a wasm build in sample visualization on the dashboard.

## Windows x64

To build netvr-cpp .dll for windows you'll need
[Visual Studio 2022](https://visualstudio.microsoft.com/downloads/) compiler
along with cmake provided by it. I recommend using the Visual Studio installer
application to install this. I would provide more detailed instructions, but
microsoft changes the download page constantly.

Older or newer version should work too, but 2022 version is what I tested with.

Open **Developer PowerShell for VS 2022**

```powershell
cd C:\Source\netvr\netvr-cpp
cmake . -G "Visual Studio 17 2022" -A x64 -B Win64
cmake --build Win64 --config Release
```

<details>
    <summary>Building from VSCode</summary>

Alternatively if you are using [vscode](https://code.visualstudio.com/) you can
use CMake Tools to build the project. If you do not have this extension it
should be recommended to you upon opening the project. The you can run
`CMake: Configure` from command palette `Ctrl+Shift+P` (it might ask a few
questions about your preferred compiler) followed by `CMake: Build`, also from
command palette.

To do this you will need to have up to date compiler installed (see above).

</details>

## Android (for Quest 2)

To build netvr-cpp .so for Quest 2 you'll need CMake. You can use the same cmake
installation from Windows build. You'll also need Android NDK, which you can get
by using [Unity Hub](https://store.unity.com/download) and installing Unity
2021.2.16f1 along with Android Build Support module.

Open powershell with CMake available, for example **Developer PowerShell for VS 2022**
and run following commands (you'll have to replace paths with correct ones for
your unity install).

```powershell
$ANDROID_NDK="D:\Stuff\UnityEditors\2021.3.19f1\Editor\Data\PlaybackEngines\AndroidPlayer\NDK"
cd C:\Source\netvr\netvr-cpp
cmake . "-DCMAKE_TOOLCHAIN_FILE=$ANDROID_NDK\build\cmake\android.toolchain.cmake" -DCMAKE_SYSTEM_NAME="Android" "-DANDROID_NDK=$ANDROID_NDK" -DANDROID_PLATFORM=android-29 -DANDROID_ABI="arm64-v8a" -B Android -GNinja
cmake --build Android
```

## Web

To build netvr-cpp .wasm for use in visualizer you'll need CMake and emscripten.
I used it under wsl, which allowed me to follow [installation instructions](https://emscripten.org/docs/getting_started/downloads.html) on their website.

You'll also need cmake, which in WSL you can install using

```bash
sudo apt install cmake ninja-build
```

Open shell with emscripten loaded and run. I recommend running it somewhere in
WSL home directory, because it becomes very slow when run on the mounted
filesystem.

```bash
mkdir -p wasm
cd wasm
emcmake cmake -DCMAKE_BUILD_TYPE=Release /mnt/c/Source/netvr/netvr-cpp -G Ninja
cmake --build .
```

The build should automatically copy the wasm and js files to correct directory
in the dashboard source tree.
