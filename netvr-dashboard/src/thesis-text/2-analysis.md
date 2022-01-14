# Analysis {#analysis}

:todo[chybí mi hard literatura, asi bych si měla sehnat tu doporučenou]

This chapter will describe various constraints and requirements that apply to this thesis. Since the target device affects most other constraints, that will be the first thing that will be chosen. The following section describes an overview of the steps which need to be done to implement a collocated VR experience. Afterwards, some steps will be expanded on and described in more detail.

## Target devices {#target-devices}

The first primary consideration when designing a VR experience is a choice of the target device. This choice will affect most other decisions because different devices have different capabilities and tracking systems.

According to Jung:cite[classification] can be divided into multiple categories. Ranging from Smartphone VR to PC-tethered, or from Stationary, through Desktop scale and Room scale to Mobility.

Of particular interest to this thesis will be devices with positional tracking as stationary users cannot interact in real-world space. Another requirement will be controllers with positional tracking so that users can see each other's hands.

The application created as a part of this thesis will therefore target most devices with 6DOF tracking, meaning that they provide an application running on it with information about its position and rotation in the real world.

The most popular:cite[steam-hardware-survey] consumer VR headset at the end of 2021 is Quest 2:cite[quest2], which means I want to support it. It features 6DOF tracking using four cameras, which it uses to reconstruct the surrounding world. Therefore, it is a good representation of an inside-out tracking system using SLAM (Simultaneous Localisation And Mapping).

Second most popular headset is Valve Index. The author of this thesis would really like to use it, but given its price and that it is not easily available through school either it will have to be omitted. As a close-enough replacement the fourth most popular, the original headset with Lighthouse tracking, HTC VIVE will be used.

Third most used PC-tethered headset is Oculus Rift S. It uses very similar tracking system to Quest 2 and was discontinued in June of 2021. For those reasons it will not be used in this thesis.

Another family of headsets is Windows Mixed Reality:footnote[Contrary to their name they have nothing to do with mixed reality and instead are virtual reality]. They are inside-out tracked similar to the Quest 2, but since they use a different implementation they might have different characteristics. Therefore if there enough time to do it I will try to run the app on one of them, but they will not be included in initial exploration.

## Required steps {#required-steps}

There are multiple moving parts required to make a collocated VR experience. The first is implementing the offline version of the experience - getting data from the headset, which is thankfully mostly handled by game engines.

The second part is to send the information between headsets. All targetted devices support IP-based networking, which makes a natural choice. Therefore either peer-to-peer or server-based networked communication has to be implemented.

Last and perhaps the most essential part is calibrating the tracking spaces of various headsets. All headsets I encountered have tracking defined as one unit being one meter, which means the output of a calibration algorithm is translation and rotation.

:todo[maybe move this to some limitations section, or reference this paragraph there]

Some headsets have a slightly wrong scale of about 0.8% according to [issue 23][1] on OpenVR Space Calibrator:cite[space-calibrator], which means that my design should be able to accommodate this. However, given that none of the headsets I have available exhibit this issue, the implementation will forgo dealing with this particular issue.

[1]: https://github.com/pushrax/openvr-spacecalibrator/issues/23

Important to note is that before the calibration can be meaningfully implemented the previous two parts have be implemented first, because the position information has to be shared between devices.

## Calibration Algorithms {#calibration-algorithms}

There are many possible options for calibrating the tracking spaces. This section will first categorise all available options.

The first option is to choose devices with a globally anchored tracking system, meaning that all devices have the same origin of their respective tracking spaces. SteamVR's Lighthouse system falls into this category. The main drawback of this option is limited options for headsets - no option to use existing devices if they do not use the correct tracking system. However, the advantage of this option is simplicity.

The second option is to fully replace the tracking system with another system, such as OptiTrack for VR:cite[optitrack-vr]. This option, once set up is as convenient as the first option and is proven by existing commercial deployments like Golem from DIVR Labs:cite[golem-vr].

:todo[ Check that Golem uses OptiTrack, but from what I read online, it seems like it]

The third option is to do a predefined set of steps once at the beginning of each session to get the calibration matrix. After this, the application relies on each devices' tracking system separately. Compared to previous options, this allows using multiple tracking systems at once. It also allows using headsets with inside-out tracking, which are usually cheaper. A disadvantage of this option is that some tracking systems drift (origin of the system moves), which causes the calibration to become incorrect over time. Another advantage of this option is weight - nothing extra needs to be attached to the system.

The fourth option is to determine the calibration continuously while the user is in VR. For example, having the HTC VIVE tracker be attached to Quest 2, calibrating the Lighthouse-based headset to the Quest. Another option to do this would be to attach OptiTrack trackers to each headset. This option is different from replacing the tracking system with a different one by still using the integrated tracking for high-frequency data and the attached system only for low-frequency calibration updates.

## Drift measurement {#drift-measurement}

To measure the drift of a tracking system and therefore the viability of using that system with one-time calibration :todo[finish]
