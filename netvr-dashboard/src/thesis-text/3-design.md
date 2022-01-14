# Implementation design {#design}

This chapter will outline the main architectural decisions taken to implement the application used as a proof of concept. As described :ref[section&nbsp;?]{title=required-steps} several interlocking parts have to be designed and subsequently implemented. This chapter will describe each of them in detail. Further, this chapter will describe simple, possible novel, calibration approach to be used as a help before implementing the fully-feature calibration.

## Communication with headset

For the engine I decided to use Unity:cite[unity]. It allows for several options on how to communicate with the headset. Unfortunately, despite the effort, it does not abstract enough for it to be complete transparent to the application. Therefore the API used to communicate with the headset has to be chosen.

Many headsets have their API - for example, VRAPI on Oculus/Meta headsets, OpenVR on headsets that use SteamVR (like HTC VIVE). More recently, headsets started supporting a unified API called OpenXR, which solves many problems with portability, but being a newer API, it has a few missing parts. But since the older APIs are getting deprecated:cite[oculus-openxr], the application will use OpenXR as the underlying API to support the chosen set of headsets.

OpenXR on the Quest 2 can be used via a Unity-provided package or an SDK from Meta directly. However, the application will not use the Oculus SDK to avoid incompatibilities with non-Oculus headsets.

The previous two decisions limit possibilities of how to display users controllers. A generic model can be used, or the application must come with a model database and detect respective controllers to guess which model to load. Finally, the application will use realistic models downloaded from WebXR Input Profiles:cite[controller-models] to improve users' perception of the calibration's correctness.

## Communication between headsets

As noted previously, applications running on different devices need to communicate with each other in real time. To achieve this a client-server architecture will be used. Disadvantage of this choice is potentially higher latency because there is one extra step in the flow of information between headsets. The advantages are greater scalability (since each device has to upload its information only once) and easier to understand information flow.

Initial implementation for the proof of concept will be done using Deno for the server runtime and WebSockets for the realtime client-server communication.

Three separate conceptual information channels will be used for various types of messages. First is time-insensitive reliable channel, which is used for exchanging infrequently changing information. For this purpose, the TCP-based WebSockets are well suited, since they have low overhead of 2-4 bytes per message and mostly only provide message chunking.

Second channel is used for real time updates. Latency is critical here, but usefulness of each message diminishes quickly over time, since it is not interesting to know where the controller was a second ago if you know where it is now. UDP is a good mechanism for this.

Third channel is used during the calibration to exchange the information needed for given calibration method. Here, reliability becomes important at the expense of latency, but latency should still be reduced to allow for more interesting calibration algorithm. This channel would be best implemented on top of UDP along with resend mechanism, but could be also implemented as a separate TCP stream to prevent head-of-line blocking with messages from the first channel.

The initial implementation uses WebSockets for all of the above for simplicity, but steps are taken to ensure that messages from each channel can be separated eventually.

### Note about compression

Size of data being uploaded to the server from each client depends on the number of devices connected and how much data is available for each device. It could be reduced by skipping redundant data (left eye is usually only transformed center eye for example). Another option to reduce the size of messages would be to compress them. For example, in one of the prototypes data upload for HTC Vive with both controllers connected used 476 bytes. If I applied Brotli compression on each message separately it would reduce the size to less or equal to 327 bytes in my experiment.

Data upload does not need to be compressed, because even with overhead of WebSockets it fits within the usual minimum MTU of 500 bytes which means that on most networks it should be transmitted as a single non-fragmented packet. Data download from the server is different because the size is the above multiplied by number of clients. So this would be a good improvement to the algorithm.

I did not implement it mostly because of simplicity of implementation since I have 3 different places of which 2 would have to implement compression and all 3 would have to implement decompression. Also the server runs in two distinct runtime environments which limits libraries available for use without complicating its build process. Lastly, bigger improvement would be achieved by moving from TCP to UDP.

## Simple calibration method

To aid with the rest of the implementation without having to implement a complex calibration routine a simple one-time calibration method was developed.

To calibrate the tracking spaces of two or more headsets the user puts each headset on top of each other. This roughly aligns positions and directions of center-eye reference frame. User then presses a button in the dashboard and the calibration is done.

The dashboard first fetches current position and rotation of all currently connected headsets. It consequently calculates transformation between the headsets while assuming that position of the ground and direction of the up vector is correct between each headset. The transformation is (assuming that y axis is up):

$$
\begin{aligned}
T_x &= \sin(r_y) \cdot z - \cos(r_y) \cdot x \\
T_y &= 0 \\
T_z &= \cos(r_y) \cdot z - \sin(r_y) \cdot x \\
R_x &= 0 \\
R_y &= -r_y \\
R_z &= 0
\end{aligned}
$$

This puts the origin of the system of coordinates under the stack of headsets. And forward vector corresponds to the direction in which the headsets were looking.

The above assumptions stem from the fact that both tested headsets have calibration procedure which sets floor and therefore translation in the y axis should be zero and from the fact that the headsets have accelerometers which means that they can measure direction of the gravity.

## Complex calibration method

One possible method for improving the calibration precision is method used by the OpenVR space calibrator:ref[space-calibrator]. This method relies on a set of pairs of controller poses. User takes one controller from one device and one controller from other device, holds them together in such a way so that they slip as little as possible. Then they move the controllers in infinity sign. The application then records the poses and from that calculates both the offset between the controllers and the offset between the tracking spaces.

The basic idea of the method is described in the math.pdf file in the source repository, but a few important details are only available through reading the source code. Code is written in C++, which can fortunately be used from unity, but is very tightly coupled to OpenVR.

::space

## Improving the calibration {#improving-the-calibration}

:todo[Run this section through grammarly once I decide which parts of it I want to keep just as a "possible future improvement" and which parts to actually implement.]

I have a few ideas on how to improve the OpenVR space calibrator (abbr. OSC) approach. First: OSC discards samples which are closer together than 20ms:footnote[Determined by reading the source code]. It would be better to choose samples in a different way. Maybe using change of position between samples, or velocity as measure by the IMU.

Furthermore OSC uses naïve system using fixed number of samples, which is great for someone who knows how to use the system, but it might be better to try to measure the error and use the result when it is good enough.

Also the offset of the tracking systems is (hopefully) going to be fixed, but the controller offset might not be even for the duration of the calibration. But if the offset shifts, it might stay the same - ie. it will be A before the shift and B after. This might be possible to counteract.

It might also be possible to use some kind of continuous calibration with error measurement, which could, in theory, allow us to do calibration without triggering it from UI. The instruction for the user would be - if you experience calibration problem, just pick up controller from other system while having your primary controller in hand, wave around and it will fix itself automatically. No having to go through an UI.

Also, the previous could be used for calibrating multiple devices together without having to go through the lengthy and confusing process of picking the controllers to calibrate, selecting them and calibrating. Just see who sees something wrong, give them other controller to wave around with and done.

It could possibly do the vive tracker/optitrack marker assignment for continuous calibration happen automatically just from having the users move around.
