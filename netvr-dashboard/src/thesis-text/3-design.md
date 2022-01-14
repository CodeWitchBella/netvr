# Implementation design {#design}

This chapter will outline the main architectural decisions taken to implement the application used as a proof of concept. As described :ref[section&nbsp;?]{title=required-steps} several interlocking parts have to be designed and subsequently implemented. This chapter will describe each of them in detail. Further, this chapter will describe simple, possible novel, calibration approach to be used as a help before implementing the fully-feature calibration.

## Communication with headset

Many headsets have their API - for example, VRAPI on Oculus/Meta headsets, OpenVR on headsets that use SteamVR (like HTC VIVE). More recently, headsets started supporting a unified API called OpenXR, which solves many problems with portability, but being a newer API, it has a few missing parts. Therefore, the application will use OpenXR as the underlying API to support the chosen set of headsets.

OpenXR on the Quest 2 can be used via a Unity-provided package or an SDK from Meta directly. However, the application will not use the Oculus SDK to avoid incompatibilities with non-Oculus headsets.

The previous two decisions limit possibilities of how to display users controllers. A generic model can be used, or the application must come with a model database and detect respective controllers to guess which model to load. Finally, the application will use realistic models to improve users' perception of the calibration's correctness.

## Communication between headsets

As noted previously, applications running on different devices need to communicate with each other in real time. To achieve this a client-server architecture will be used. Disadvantage of this choice is potentially higher latency because there is one extra step in the flow of information between headsets. The advantages are greater scalability (since each device has to upload its information only once) and easier to understand information flow.

Initial implementation for the proof of concept will be done using Deno for the server runtime and WebSockets for the realtime client-server communication.

Three separate conceptual information channels will be used for various types of messages. First is time-insensitive reliable channel, which is used for exchanging infrequently changing information. For this purpose, the TCP-based WebSockets are well suited, since they have low overhead of 2-4 bytes per message and mostly only provide message chunking.

Second channel is used for real time updates. Latency is critical here, but usefulness of each message diminishes quickly over time, since it is not interesting to know where the controller was a second ago if you know where it is now. UDP is a good mechanism for this.

Third channel is used during the calibration to exchange the information needed for given calibration method. Here, reliability becomes important at the expense of latency, but latency should still be reduced to allow for more interesting calibration algorithm. This channel would be best implemented on top of UDP along with resend mechanism, but could be also implemented as a separate TCP stream to prevent head-of-line blocking with messages from the first channel.

The initial implementation uses WebSockets for all of the above for simplicity, but steps are taken to ensure that messages from each channel can be separated eventually.

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

The above assumptions stem from the fact that both tested headsets have calibration procedure which sets floor and therefore translation in the y axis should be zero and from the fact that
