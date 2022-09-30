PSREC
=====

psrec is a stand-alone (at least for recording) command line application designed for recording basic aspects of other applications that are run, currently CPU usage and memory usage over time.

It's essentially a compiled (using Rust) application very similar in use-case to psrecord.

Currently visualisation of the data needs to be done externally via a provided Python script which uses matplotlib, and the main psrec executable can save the results to a .CSV file.

Usage
=====

psrec has two main modes of interacting with the application to be monitored: it can start an application to record (with optional arguments), or it can attach to an existing process based off the PID.

General command line args are:

    ./psrec [optional options] --export <path_to_save_file.csv> <mode>

Additional option args that are supported:

    * `--interval <2>`: Set the interval period in seconds between each recording (default is 1 second).
    * `--print-values`: Print out the recorded values to stderr live as they're sampled from the process.
    * `--export <path_to_save_file.csv>`: Save the recorded results to this file.
    * `--absolute-cpu-usage`: If specified, psrec will not normalise the CPU usage sample values to the number of threads on the machine. By default it does, and full all thread CPU usage will be 100.0. With this option set, it will be 100.0 x number of threads. 

Attaching to an existing process
--------------------------------

    ./psrec --export <path_to_save_results.csv> attach <PID>

This will attempt to attach to the process with the provided Process ID, and start recording the CPU usage and current RSS memory usage every second by default. After the attached process has finished, results will be saved to the file path provided by the `--export` command line arg.

Starting a new process
----------------------

    ./psrec --export <path_to_save_results.csv> start <path_to_application> [optional_command line args]

This will attempt to spawn off the specified process (with optional command line args to that process), and start recording the CPU usage and current RSS memory usage every second by default. After the process has finished, results will be saved to the file path provided by the `--export` command line arg.


Visualising Results
===================

Results are currently saved to .csv file (more formats will likely be supported in the future), and a provided Python script can be used to visualise them in chart form.

Running:

    python3 psrec_gen_plot.py <path_to_results_file.csv>

With Python 3 with matplotlib libs installed, should display a chart of the results.
