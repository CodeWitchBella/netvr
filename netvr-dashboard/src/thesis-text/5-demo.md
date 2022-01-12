# Demo application

To demonstrate all approaches and concepts in previous sections a simple application was implemented.

::space

**List of implemented stuff**

- networked synchronization of VR devices (controllers and headsets)
- loading of appropriate models for each headset and controller (gltf)
- simple calibration routine
- applying calibration matrix
- should work for more than 2 headsets too (untested)

::space

**List of WIP stuff**

- complex calibration method. Cpp compiles, loads, functions can be called, relevant parts of code are copy-pasted. It needs to have a UI for selecting controllers and starting the calibration. Also probably higher-frequency, separately-sent calibration data with timestamps so that it does not depend on realtime synchronisation implementation.

::space

**List of completely missing stuff**

- Time sync (simple or maybe even latency aware)
- Something for a user to do so that I can describe actions with people as "walking around a thing" in the text.
- Interesting would be to actually have a VRPN client and allow to replace tracking with optitrack's (this would also simplify some measurements). But it seems to be too much work to be worth it.

:todo[take a picture of multiple users in VR world]

## Dashboard {#demo-dashboard}

To allow testing of implemented algorithm as well as to simplify development a web-based dashboard was implemented. It can be seen on figure :ref[web-dashboard]. It implements the full communication protocol except for sending device data.

![Web-based dashboard as of 2022-01-10](web-dashboard.png 'web-dashboard')

## Images not referenced from text

![Unity network drawer](unity-net-drawer.png 'net-drawer')

![Unity device drawer](unity-device-drawer.png 'device-drawer')
