# Analysis

This chapter will describe various constraints and requirements that I operate under. :todo[describe the actual contents of this chapter] First I'll do X, then Y and the finish it off with Z.

## Target devices

First major consideration is a choice of target device. This will affect most other technical decisions, because different devices have different capabilities and tracking systems. In this thesis I want to target most devices which have 6DOF tracking, meaning that they provide application running on it with information about its position and rotation in real world. Many popular devices in this cathegory also have two 6DOF-tracked controllers, so I'll assume that as well.

Currently most popular consumer VR headset is Quest 2, which means that I definitelly want to support it. It features 6DOF tracking using four cameras, which it uses to reconstruct surrounding world. This makes it a good representation of inside-out tracking system with use of SLAM (Simultaneous Localization And Mapping).

:todo[write about Vive]

## What needs to be done

:todo[Better heading for this section]

There are multiple moving parts are required to make a collocated VR experience. First is implementing the offline version of the experience - getting data from the headset. This is thankfully mostly handled by game engines.

Second part is to send the information between headsets. All recent headsets which support 6DOF

## Takeouts

:todo[Move this to implementation]

Many headsets have their own API - for example OVR on Oculus/Meta headsets, OpenVR on headsets which use SteamVR (like HTC Vive). More recently headsets started supporting unified API called OpenXR, which solves a lot of problems with portability, but being newer API it has few missing parts.
