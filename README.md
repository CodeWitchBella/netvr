# netvr

Diplomka thingo, WIP

Unity version: see [ProjectVersion.txt](netvr-unity/ProjectSettings/ProjectVersion.txt)

Plugins: none, only OpenXR, because everything else breaks on some headset

## Update controller models

```bash
cd netvr-unity
curl `npm view @webxr-input-profiles/assets dist.tarball` -o pkg.tgz
tar xf pkg.tgz
mkdir -p Assets/Resources
rm -rf Assets/Resources/Controllers
mv package/dist/profiles Assets/Resources/Controllers
rm -rf pkg.tgz package
```
