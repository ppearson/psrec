TODO
====

Possible Issues
---------------

* When capturing on Linux with the ProcessSamplerAdvanced sampler, the first captured CPU usage
  sample value at elapsed time 0 is often wrong (too big).

Future enhancements
-------------------

(Likely)

* Add ability to record other process details like user/system time breakdown, etc
* Config file support - with preset support, to make recording more user-friendly?
* Writing the record sample values out live to file as they're sampled, rather than at the end (maybe optionally?).

(Possible, but lower-priority)

* Better support on non-Linux platforms.
* Recording of other process metrics, like Disk/Network I/O.
* Other additional means of visualising data: gnuplot export? built-in visualisation (Plotters crate)?
* Other export file formats. (json?)
