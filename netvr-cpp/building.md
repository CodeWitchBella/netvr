# Building netvr-cpp

## Windows x64

Open **Developer PowerShell for VS 2022**

```powershell
cd C:\Source\netvr\netvr-cpp
cmake . -G "Visual Studio 17 2022" -A x64 -B Win64
cmake --build Win64 --config Release
```

## Android (for Quest 2)

Open **Developer PowerShell for VS 2022** (or anything else with CMake, here I
use NDK contained within Unity installation).

```powershell
$ANDROID_NDK="C:\Stuff\Unity Editors\2021.2.3f1\Editor\Data\PlaybackEngines\AndroidPlayer\NDK"
cmake . "-DCMAKE_TOOLCHAIN_FILE=$ANDROID_NDK\build\cmake\android.toolchain.cmake" -DCMAKE_SYSTEM_NAME="Android" "-DANDROID_NDK=$ANDROID_NDK" -DANDROID_PLATFORM=android-29 -DANDROID_ABI="arm64-v8a" -B Android -GNinja
cmake --build Android
```
