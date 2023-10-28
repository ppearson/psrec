Version 0.9
-----------

* Added support for writing out metadata about the recording params and host hardware (thread count
  currently) to the .csv output files as comments, so as to be able to more robustly know if
  the CPU usage numbers represent normalised or absolute values when plotting the chart values.
* Fixed issue with the setting for recording normalised CPU values being initialised incorrectly.
* Fixed issue with psrec not correctly recognising that attaching to a process, starting a process or
  creating a process sampler had failed, and so not reporting an error, and creating an empty
  export results file anyway for no reason.