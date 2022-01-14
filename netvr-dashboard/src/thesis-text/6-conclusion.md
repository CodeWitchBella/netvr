# Conclusion {#conclusion}

Over the course of writing this project multiple options of how to implement a collocated VR experience were considered. One option was followed and the result is working demonstration application which can be used as a basis for further development. I also created 3D printed Quest 2 to OptiTrack adapter rig, which is useful for various measurements.

## Future improvement

As evidenced by previous chapters the direction of further improvement is to implement the complex calibration algorithm fully. Then its precision needs to be measured and once that is done I can try to improve it with changes listed in :ref[section ?]{title=improving-the-calibration}. I also want to measure the Quest 2's drift.

Another avenue for improvement is to lower the latency of used network solution. It is also possible to calculate the impact of latency on incorrectly-implemented calibration algorithm, which could be useful as a guide for anyone trying to implement the calibration in the future.

If I can't meaningfully improve the calibration it might be interesting to implement more calibration methods such as using hand tracking:cite[slam-hands]

Lastly, I would like to keep an eye out for updates from Meta to Quest's firmware because it might become possible to implement something similar to what HTC announced recently:cite[focus-lbe]. Or their might release their own solution:cite[quest-mapsharing] making this whole project basically useless.
