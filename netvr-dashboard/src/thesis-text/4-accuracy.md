# Accuracy Measurement

:todo[drop personal usage of personal pronoun from this section - should I prefer passive voice, or using stuff like "this thesis" and "author of this thesis"?]

While the calibration subjectively works, measuring how well it works is also necessary. Another consideration, as previously noted, is whether and for how long is it possible to expect the calibration to hold if a one-time method of calibration is used.

## Quest 2 drift {#quest-2-drift}

First, I measured how much does Quest 2 change its tracking space over time using OptiTrack as a ground-truth reference. To do so, I designed 3D printed rig (figure :ref[quest2-optitrack]) with an OptiTrack marker so that my measurements are repeatable without using adhesives for attaching the markers.

![Quest 2 with 3D printed OptiTrack marker rig attached.](quest2-optitrack.png 'quest2-optitrack')

:todo[maybe take a better picture? Maybe take a picture from leap motion showing the retroreflective markers. Škoda že není černý, takhle se moc nedá vyfotit na světlém pozadí 😢]

OptiTrack is set to track a group of markers as a rigid body to measure the drift. Then I started a simple application for logging the headset's position. To avoid _unintended_ headset sleep, I covered the presence sensor for the entire duration of the measurement. The data is then collected and analysed after the fact.

Measured actions always started with the headset being set on a chair in the middle of the room without moving, then I put the headset on my head and performed one of following actions:

1. walking around the room slowly,
2. slow movement in place (keeping feet in the same place),
3. fast movement in place,
4. fast movement while waving hands in front of the headset's cameras in place,
5. pacing around the room while waving hands in front of the headset

I also tried putting the headset to sleep and waking it up again, both in the identical and in a different location, to see if it affected the drift. Finally, I tried occluding headsets cameras fully, moving to a different location and uncovering them to see if loss of tracking introduces drift.

:todo[I'll probably replace the rest of this section once I have the data. If not I should run it through grammarly]

To do the computation I first aligned the datasets' timestamps by using the pause at the start. To do that I computed speed of both optitrack and quest 2 output and looked for long period of time containing only zeros. This was quiet simple.

Then to see if the data is reasonable I computed "distance from start" and plotted it (figure :ref[chart]). The output is not reasonable. I suspect that one of the datasets has a wrong scale, but it does not seem to be usable. Therefore I'll have to figure out the source of the problem and redo the measurement. Also, optitrack was loosing tracking constantly while measuring - this can be fixed by adding more cameras, but I did not want to spend too much time on acquiring data of unknown quality (I did not have the tools for analysis ready because I did not know the output format).

I am still not sure how to calculate the drift. Following articles will might have an answer:cite[vive-analysis]:cite[vive-analysis2].

![Chart showing the wrong data](chart.svg 'chart')

## Simple method calibration precision {#simple-method-calibration-precision}

:todo[I still do not know how to measure this? Probably assume that complex method works well if the controllers are attached well, calibrate everything to OptiTrack and then measure A->OT->B vs A->(simple)->B]

## Complex method calibration precision {#complex-method-calibration-precision}

:todo[same methodology as in above?]

## Effect of latency on calibration result {#effect-of-latency-on-calibration-result}

My implementation deals with _symmetric_ latency correctly, but if one were to omit such code or the latency was asymmetric, it would affect the calibration results of the complex method. Therefore, this section estimates the error with respect to the speed at which the user moves tracked devices and subsequently validates the estimate with real-world measurement.

:todo[actually do the thing]
