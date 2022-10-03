TODO
====

Future enhancements
-------------------

(Likely)

* Implmement support for killing processes with Ctrl+C.
* Add ability to record sample details of child processes of the main process.
* Config file support - with preset support, to make recording more user-friendly?
* Checking any export file path is valid and write-able before starting.
* Writing the record sample values out live to file as they're sampled, rather than at the end (maybe optionally?).

(Possible, but lower-priority)

* Recording of other process metrics, like Disk/Network I/O.
* Other additional means of visualising data: gnuplot export? built-in visualisation (Plotters crate)?
* Other export file formats. (json?)
