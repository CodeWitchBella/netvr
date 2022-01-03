# Accuracy Measurement

While the calibration subjectively works, it is also important to measure how well it works. Another consideration as previously noted is whether and for how long is it possible to expect the calibration to hold if one-time method of calibration is used.

## Quest 2 drift

First I measured how much does Quest 2 change its tracking space over time using OptiTrack as ground-truth reference. To do so I designed 3D printed rig (figure :ref[quest2-optitrack]) with OptiTrack marker so that my measurements are repeatable without using adhesives for attaching the markers.

![Quest 2 with 3D printed OptiTrack marker rig attached.](quest2-optitrack.png 'quest2-optitrack')

:todo[maybe take a better picture?]

To measure the drift OptiTrack is set to track group of markers as rigid body, then I started simple application for logging headset's position. To avoid _unintended_ headset sleep I covered the presence sensor for the whole time. The data is then collected and analysed after-the-fact.

Measured actions always started with headset being set on a chair in the middle of the room without moving, then I put the headset on my head and performed the action. Actions were:

1. walking around the room slowly,
2. slow movement in place (keeping feet in the same place),
3. fast movement in place,
4. fast movement while waving hands in front of the headset's cameras in place,
5. pacing around the room while waving hands in front the headset

I also tried putting headset to sleep and waking it up again both in the same place and in different place to see if it affects the drift. Finally, I tried occluding headsets cameras fully, moving to different place and uncovering them to see if loss of tracking introduces drift.

To analyse the data I did the following: first I matched samples from OptiTrack and Quest 2 as best as possible according to their timestamps. Then from the stable initial position I determined initial offset between OptiTrack's rigid body and Quest 2's center eye.

```tex
$$P^0_{quest}&=P^0_{rb} + P^{0}\\
R^0_{quest}&=R^0_{rb} + R^{0}$$
```

:todo[fix equation to actually be equation]

:todo[rotation equation does not make sense]

:todo[describe the variables in the equation]

Afterwards I calculated the same offset for each subsequent matched pair and subtracted the initial offset. This provides the cumulative drift.

:todo[insert chart once I have one - definitely time/error, maybe distance traveled/error?]

:todo[I should probably reference some article which calculates the same, maybe some VIVE evaluations]

## Simple method calibration precision

:todo[I still don't know how to measure this? Probably assume that complex method works well if the controllers are attached well, calibrate everything to OptiTrack and then measure A-OT-B vs A-(simple)-B]

## Complex method calibration precision

:todo[same methodology as in above?]

## Effect of latency on calibration result

My implementation deals with _symmetric_ latency correctly, but if one were to omit such code or the latency was asymmetric it would affect the calibration results of the complex method. This section estimates the error with respect to speed in which user moves tracked devices and subsequently validates the estimate with real-world measurement.

:todo[actually do the thing]
