TODO
====

Future enhancements
-------------------

(Likely)

* Add ability to record other process details like user/system time breakdown, number of active threads, etc
* Config file support - with preset support, to make recording more user-friendly?
* Checking any export file path is valid and write-able before starting.
* Writing the record sample values out live to file as they're sampled, rather than at the end (maybe optionally?).

(Possible, but lower-priority)

* Recording of other process metrics, like Disk/Network I/O.
* Other additional means of visualising data: gnuplot export? built-in visualisation (Plotters crate)?
* Other export file formats. (json?)
