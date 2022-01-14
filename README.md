# netvr

Diplomka thingo, WIP

Unity version: see [ProjectVersion.txt](netvr-unity/ProjectSettings/ProjectVersion.txt)

Plugins: none, only OpenXR, because everything else breaks on some headset

## Project structure

- **isbl-thesis** not part of the thesis, only for generating the pdf
- **netvr-server** server used for communicating between various clients
- **netvr-dashboard** html/javascript application used to see what clients are connected in one place, also used to trigger calibration
- **netvr-unity** main VR application used as a demonstration of implemented technologies
- **netvr-cpp** used from unity, whenever something is easier to implement in cpp than in C# it goes here.
- **schemas** json schemas describing various transmitted JSONs. Used mostly for documentation purposes.
- **.github** contains definitions for building everything in this project in automatic manner. Only part which is not automated is android build of netvr-unity, because I could not figure that out.

See README in each folder for instructions on how to build each part.

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
