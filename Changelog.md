Version 0.9.2
-------------

* Added check to verify that the directory path of any export results file to be written exists
  before recording starts.

Version 0.9.1
-------------

* Fixed an issue with the error message that's printed when starting the recorded process failed contains
  the wrong string in the case that the start command was a quoted string with additional arguments.

Version 0.9
-----------

* Added support for writing out metadata about the recording params and host hardware (thread count
  currently) to the .csv output files as comments, so as to be able to more robustly know if
  the CPU usage numbers represent normalised or absolute values when plotting the chart values.
* Fixed issue with the setting for recording normalised CPU values being initialised incorrectly.
* Fixed issue with psrec not correctly recognising that attaching to a process, starting a process or
  creating a process sampler had failed, and so not reporting an error, and creating an empty
  export results file anyway for no reason.
* Made psrec_gen_plot.py plotting Python script have a hashbang for easier execution.
* Fixed some robustness issues with psrec_gen_plot.py plotting file, mainly when there was no valid
  plot data to show.
* Added support for splitting the Start 'command' argument after a space char into further arguments for
  the command to be started, so as to work around a limitation with the argh crate, and allow passing
  through arbitrary commands to the processes to be started by specifying a single space-separated
  command to run plus any optional arguments as a quoted 'command' argument.
