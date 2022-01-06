# Accuracy Measurement

:todo[drop personal usage of personal pronoun from this section - should I prefer passive voice, or using stuff like "this thesis" and "author of this thesis"?]

While the calibration subjectively works, measuring how well it works is also necessary. Another consideration, as previously noted, is whether and for how long is it possible to expect the calibration to hold if a one-time method of calibration is used.

## Quest 2 drift

First, I measured how much does Quest 2 change its tracking space over time using OptiTrack as a ground-truth reference. To do so, I designed 3D printed rig (figure :ref[quest2-optitrack]) with an OptiTrack marker so that my measurements are repeatable without using adhesives for attaching the markers.

![Quest 2 with 3D printed OptiTrack marker rig attached.](quest2-optitrack.png 'quest2-optitrack')

:todo[maybe take a better picture?]

OptiTrack is set to track a group of markers as a rigid body to measure the drift. Then I started a simple application for logging the headset's position. To avoid _unintended_ headset sleep, I covered the presence sensor for the entire duration of the measurement. The data is then collected and analysed after the fact.

Measured actions always started with the headset being set on a chair in the middle of the room without moving, then I put the headset on my head and performed one of following actions:

1. walking around the room slowly,
2. slow movement in place (keeping feet in the same place),
3. fast movement in place,
4. fast movement while waving hands in front of the headset's cameras in place,
5. pacing around the room while waving hands in front of the headset

I also tried putting the headset to sleep and waking it up again, both in the identical and in a different location, to see if it affected the drift. Finally, I tried occluding headsets cameras fully, moving to a different location and uncovering them to see if loss of tracking introduces drift.

I did the following to analyse the data: first, I matched samples from OptiTrack and Quest 2 as best as possible according to their timestamps. Then from the stable initial position, I determined the initial offset between OptiTrack's rigid body and Quest 2's centre eye.

```tex
$$P^0_{quest}&=P^0_{rb} + P^{0}\\
R^0_{quest}&=R^0_{rb} + R^{0}$$
```

:todo[fix equation to be the actual equation used]

:todo[rotation equation does not make sense]

:todo[describe the variables in the equation]

Afterwards, I calculated the same offset for each subsequent matched pair and subtracted the initial offset. This computation provides the cumulative drift.

:todo[insert chart once I have one - certainly time/error, maybe distance travelled/error?]

:todo[I should probably reference some article which calculates the same, maybe some VIVE evaluations]

## Simple method calibration precision

:todo[I still do not know how to measure this? Probably assume that complex method works well if the controllers are attached well, calibrate everything to OptiTrack and then measure A-OT-B vs A-(simple)-B]

## Complex method calibration precision

:todo[same methodology as in above?]

## Effect of latency on calibration result

My implementation deals with _symmetric_ latency correctly, but if one were to omit such code or the latency was asymmetric, it would affect the calibration results of the complex method. Therefore, this section estimates the error with respect to the speed at which the user moves tracked devices and subsequently validates the estimate with real-world measurement.

:todo[actually do the thing]
