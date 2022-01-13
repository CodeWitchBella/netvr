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

## Improving the calibration {#improving-the-calibration}

:todo[Run this section through grammarly once I decide which parts of it I want to keep just as a "possible future improvement" and which parts to actually implement.]

I have a few ideas on how to improve the OpenVR space calibration (OSC) approach. First: OSC discards samples which are closer together than 20ms. It would be better to choose samples in a different way. Maybe position, or velocity?

Furthermore OSC uses naive system using fixed number of samples, which is great for someone who knows how to use the system, but it might be better to try to measure the error and use the result when it is good enough.

Also the offset of the tracking systems is (hopefully) going to be fixed, but the controller offset might not be even for the duration of the calibration. But if the offset shifts, it might stay the same - ie. it will be A before the shift and B after. This might be possible to counteract.

It might also be possible to use some kind of continuous calibration with error measurement, which could, in theory, allow us to do calibration without triggering it from UI. The instruction for the user would be - if you experience calibration problem, just pick up controller from other system while having your primary controller in hand, wave around and it will fix itself ✨magically✨. No having to go through confusing UI.

Also, the previous could be used for calibrating multiple devices together without having to go through the lengthy and confusing process of picking the controllers to calibrate, selecting them and calibrating. Just see who sees something wrong, give them other controller to wave around with and done.

It could possibly do the vive tracker/optitrack marker assignment for continuous calibration happen automatically just from having the users move around.
