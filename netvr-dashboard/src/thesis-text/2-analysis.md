# Analysis

This chapter will describe various constraints and requirements that apply to this thesis. :todo[describe the actual contents of this chapter] First, I will do X, then Y and then finish it with Z.

## Target devices

The first primary consideration is a choice of the target device. This choice will affect most other technical decisions because different devices have different capabilities and tracking systems. This thesis will target most devices with 6DOF tracking, meaning that they provide an application running on it with information about its position and rotation in the real world. In addition, this thesis will rely on the presence of two 6DOF-tracked controllers since most devices in this category have them available by default.

The most popular:cite[steam-hardware-survey] consumer VR headset in 2021 is Quest 2:cite[quest2], which means I want to support it. It features 6DOF tracking using four cameras, which it uses to reconstruct the surrounding world. Therefore, it is a good representation of an inside-out tracking system using SLAM (Simultaneous Localisation And Mapping).

:todo[write about VIVE]

:todo[list other headsets in vrlab, because I want to try them all (except the lighthouse ones, one of lighthouse two should be enough)]

## Required steps

:todo[Better heading for this section]

There are multiple moving parts required to make a collocated VR experience. The first is implementing the offline version of the experience - getting data from the headset, which is thankfully mostly handled by game engines.

The second part is to send the information between headsets. All recent headsets which support 6DOF :todo[finish this paragraph]

Last and perhaps the most essential part is calibrating the tracking spaces of various headsets. All headsets I encountered have tracking defined as one unit being one meter, which means the output of a calibration algorithm is translation and rotation.

:todo[maybe move this to some limitations section, or reference this paragraph there]

Some headsets have a slightly wrong scale of about 0.8% according to [issue 23][1] on OpenVR Space Calibrator:cite[space-calibrator], which means that my design should be able to accommodate this. However, given that none of the headsets I have available exhibit this issue, the implementation will forgo dealing with this particular issue.

[1]: https://github.com/pushrax/openvr-spacecalibrator/issues/23

## Calibration Algorithms

There are many possible options for calibrating the tracking spaces. This section will first categorise all available options.

The first option is to choose devices with a globally anchored tracking system, meaning that all devices have the same origin of their respective tracking spaces. SteamVR's Lighthouse system falls into this category. The main drawback of this option is limited options for headsets - no option to use existing devices if they do not use the correct tracking system. However, the advantage of this option is simplicity.

The second option is to fully replace the tracking system with another system, such as OptiTrack for VR:cite[optitrack-vr]. This option, once set up is as convenient as the first option and is proven by existing commercial deployments like Golem from DIVR Labs:cite[golem-vr].

:todo[ Check that Golem uses OptiTrack, but from what I read online, it seems like it]

The third option is to do a predefined set of steps once at the beginning of each session to get the calibration matrix. After this, the application relies on each devices' tracking system separately. Compared to previous options, this allows using multiple tracking systems at once. It also allows using headsets with inside-out tracking, which are usually cheaper. A disadvantage of this option is that some tracking systems drift (origin of the system moves), which causes the calibration to become incorrect over time. Another advantage of this option is weight - nothing extra needs to be attached to the system.

The fourth option is to determine the calibration continuously while the user is in VR. For example, having the HTC VIVE tracker be attached to Quest 2, calibrating the Lighthouse-based headset to the Quest. Another option to do this would be to attach OptiTrack trackers to each headset. This option is different from replacing the tracking system with a different one by still using the integrated tracking for high-frequency data and the attached system only for low-frequency calibration updates.

## Drift measurement

To measure the drift of a tracking system and therefore the viability of using that system with one-time calibration :todo[finish]

## Improving the calibration

:todo[Run this section through grammarly once I decide which parts of it I want to keep just as a "possible future improvement" and which parts to actually implement.]

I have a few ideas on how to improve the OpenVR space calibration (OSC) approach. First: OSC discards samples which are closer together than 20ms. It would be better to choose samples in a different way. Maybe position, or velocity?

Furthermore OSC uses naive system using fixed number of samples, which is great for someone who knows how to use the system, but it might be better to try to measure the error and use the result when it is good enough.

Also the offset of the tracking systems is (hopefully) going to be fixed, but the controller offset might not be even for the duration of the calibration. But if the offset shifts, it might stay the same - ie. it will be A before the shift and B after. This might be possible to counteract.

It might also be possible to use some kind of continuous calibration with error measurement, which could, in theory, allow us to do calibration without triggering it from UI. The instruction for the user would be - if you experience calibration problem, just pick up controller from other system while having your primary controller in hand, wave around and it will fix itself ✨magically✨. No having to go through confusing UI.

Also, the previous could be used for calibrating multiple devices together without having to go through the lengthy and confusing process of picking the controllers to calibrate, selecting them and calibrating. Just see who sees something wrong, give them other controller to wave around with and done.

It could possibly do the vive tracker/optitrack marker assignment for continuous calibration happen automatically just from having the users move around.
