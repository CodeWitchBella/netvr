# Analysis

This chapter will describe various constraints and requirements that I operate under. :todo[describe the actual contents of this chapter] First I'll do X, then Y and the finish it off with Z.

## Target devices

First major consideration is a choice of target device. This will affect most other technical decisions, because different devices have different capabilities and tracking systems. In this thesis I want to target most devices which have 6DOF tracking, meaning that they provide application running on it with information about its position and rotation in real world. Many popular devices in this category also have two 6DOF-tracked controllers, so I'll assume that as well.

Currently, most popular:cite[steam-hardware-survey] consumer VR headset is Quest 2:cite[quest2], which means that I definitely want to support it. It features 6DOF tracking using four cameras, which it uses to reconstruct surrounding world. This makes it a good representation of inside-out tracking system with use of SLAM (Simultaneous Localization And Mapping).

:todo[write about VIVE]

:todo[list other headsets in vrlab, because I want to try them all (except the lighthouse ones, one of lighthouse 2 gen should be enough)]

## What needs to be done

:todo[Better heading for this section]

There are multiple moving parts are required to make a collocated VR experience. The first is implementing the offline version of the experience - getting data from the headset. This is thankfully mostly handled by game engines.

The second part is to send the information between headsets. All recent headsets which support 6DOF :todo[finish this paragraph]

Last and perhaps the most interesting part is to calibrate tracking spaces of various headsets. All headsets I encountered have tracking defined as one unit being one meter, which means that we only have to determine translation and rotation.

:todo[maybe move this to some limitations section, or just reference this paragraph there]

Some headsets have a slightly wrong scale of about 0.8% according to [issue 23][1] on OpenVR Space Calibrator:cite[space-calibrator], which means that my design should be able to accommodate this. But given that none of the headsets I have available exhibit this issue I won't be handling this in the implementation.

[1]: https://github.com/pushrax/openvr-spacecalibrator/issues/23

## Calibration Algorithms

There are many possible options how to calibrate the tracking spaces. First I would like to categorize the available options.

The first option is to choose devices with a tracking system, which is globally anchored between devices. SteamVR's Lighthouse system falls into this category. Main drawback of this option is limited options for headsets - no option to use existing devices, if they do not use correct tracking system. Advantage of this option is simplicity.

Second option is to fully replace the tracking system with other system, for example using OptiTrack for VR:cite[optitrack-vr]. This option, once setup, is as convenient as the first option, and is proven by existing commercial deployments like Golem from DIVR Labs:cite[golem-vr].

:todo[ Check that Golem actually uses OptiTrack, but from what I read online it seems like it]

Third option is to do a predefined set of steps once, at the beginning of session to get the calibration matrix. After this, the application relies on each devices' tracking system separately. Compared to previous options, this allows using multiple tracking systems at once. It also allows using headsets with inside-out tracking, which are usually cheaper. A disadvantage of this option is that some tracking systems drift (origin of the system moves), which causes the calibration to become incorrect over time. Another advantage of this option is weight - nothing extra needs to be attached to the system.

Fourth option is to determine the calibration continuously while the user is in VR. This can be accomplished for example by attaching HTC VIVE tracker to Quest 2, which would calibrate Lighthouse-based headset to the Quest. Another option of how to do this would be to attach OptiTrack trackers to each headset. This option is different from option 2 by still using integrated tracking for high-frequency data and only using the attached system for low-frequency calibration updates.

## Drift measurement

To measure drift of a tracking system and therefore viability of using that system with one-time calibration :todo[finish]
