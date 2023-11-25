/*
 psrec
 Copyright 2022-2023 Peter Pearson.
 Licensed under the Apache License, Version 2.0 (the "License");
 You may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 http://www.apache.org/licenses/LICENSE-2.0
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
 ---------
*/

mod process_sampler;

#[cfg(target_os = "linux")]
mod process_sampler_advanced;

mod process_samples;
mod process_recorder;
mod utils;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use argh::FromArgs;

use crate::process_recorder::*;
use crate::process_samples::ProcessRecording;

// TODO: this is pretty masochistic just to print a help banner/message formatted somewhat as I want it,
//       it's probably worth using another command line parser crate which allows better flexibility,
//       but this is one of the smallest code-size ones I've found...
#[derive(FromArgs)]
#[argh(description = r#"psrec 0.9.1.
Copyright 2022-2023 Peter Pearson.

A utility to record information about a process' execution statistics, e.g. cpu and memory usage."#,
       example = r#"psrec -i 250ms -c -e /tmp/outfile1.csv attach <pid>
psrec -i 5s -c -e /tmp/outfile2.csv start <command>
psrec -i 250ms -c -e /tmp/outfile3.csv start "<command_to_run> [additional_arg1] [additional_arg2]""#)
]
struct MainArgs {
    #[argh(subcommand)]
    command: SubCommandEnum,

    #[argh(option, short = 'i')]
    /// interval between each sample of the process in various suffix units (ms/s/m/h). Default is 1 second.
    /// Not specifying a suffix unit char will use seconds.
    interval: Option<String>,

    #[argh(option, short = 'd')]
    /// duration to record for in various suffix units (s/m/h). By default will be until the process being recorded ends.
    /// Not specifying a suffix unit char will use seconds.
    duration: Option<String>,
 
    /// whether to use absolute (wallclock) times instead of time elapsed after start
//    #[argh(switch)]
//    absolute_timestamps: bool,

    /// whether to record cpu usage as 'normalised' values (out of 100%), instead of Absolute values (default).
    /// Absolute values will scale over 100.0 for the number of threads, so 800.0 will be 8 threads using full CPU.
    /// Normalised will be normalised to 100.0, so instead of 800.0 in the above example, it will be 100.0,
    /// and 1 thread using 100% CPU will be 15.0% (assuming the computer has 8 cores/threads).
    /// 
    /// Note: this will only work in the most basic of scenarios: i.e. where std::thread::available_parallelism()
    /// returns the number of threads the process being recorded will be running on. The normalised value will likely
    /// be incorrect in other more complex scenarios, e.g. running under cgroups environments that mask/limit the CPU
    /// cores a process can run on for example.
    #[argh(switch, short = 'n')]
    normalise_cpu_usage: bool,

    /// whether to record the data for child processes as well as the main process. Defaults to off (false).
    #[argh(switch, short = 'c')]
    record_child_processes: bool,

    /// whether to record the current thread count of the process
    #[argh(switch, short = 't')]
    record_thread_count: bool,

    /// whether to print out values live as process is being recorded to stderr
    #[argh(switch)]
    print_values: bool,

    #[argh(option, short = 'e')]
    /// file path to export/save raw sample data to. File type is detected from the file extension.
    /// Only .csv format is supported currently.
    export: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommandEnum {
    // Start the specified process with optional command line args
    Start(SubCommandStart),
    // Attach to a process based off the provided process ID
    Attach(SubCommandAttach),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Start a process with command line args.
#[argh(subcommand, name = "start")]
struct SubCommandStart {
    #[argh(positional)]
    /// command line command to start/run and record
    command: String,

    #[argh(positional, greedy)]
    /// command line args
    args: Vec<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Attach to an existing process.
#[argh(subcommand, name = "attach")]
struct SubCommandAttach {
    #[argh(positional)]
    /// PID of process to attach to and record
    pid: u32,
}


fn main() {
    let args: MainArgs = argh::from_env();

    let mut record_params = ProcessRecordParams::new(args.interval, args.duration);

    // TODO: something better than this... pass in args to ProcessRecordParams?
    //       or at least encapsulate it somewhere...
    if args.normalise_cpu_usage {
        record_params.set_normalise_cpu_usage(args.normalise_cpu_usage);
    }
    if args.print_values {
        record_params.set_print_values(true);
    }
    if args.record_child_processes {
        record_params.set_record_child_processes(true);
    }
    if args.record_thread_count {
        record_params.set_record_thread_count(true);
    }

    let mut recording_results: Option<ProcessRecording> = None;

    // variable to allow interrupting with Ctrl+C handler...
    let has_been_cancelled_flag = Arc::new(AtomicBool::new(false));

    // register Ctrl+C handler...
    let cancel_atomic = has_been_cancelled_flag.clone();
    let res = ctrlc::set_handler(move || {
        // set our atomic boolean to true, signalling we want to exit the recording loop
        cancel_atomic.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    if let Err(err) = res {
        eprintln!("Warning: Could not register Ctrl+C interrupt handler: {}", err);
        // TODO: maybe we want to abort instead? Or use command line args to control any logic here?
    }

    if let SubCommandEnum::Attach(attach) = args.command {
        eprintln!("Attaching to process PID: {}...", attach.pid);

        let recorder: Option<ProcessRecorderAttach> = ProcessRecorderAttach::new(attach.pid, &record_params);
        if recorder.is_none() {
            eprintln!("Error attaching to process...");
            return;
        }

        let mut recorder: ProcessRecorderAttach = recorder.unwrap();
        // Note: start() prints some progress...
        if !recorder.start(has_been_cancelled_flag) {
            // failed to create sampler, so error out...
            return;
        }

        recording_results = Some(recorder.get_recording());

        eprintln!("Attached process has exited.");
    }
    else if let SubCommandEnum::Start(start) = args.command {
        // first of all, see if 'start.command' has spaces in: if so, it's almost certainly a quoted string command line
        // arg, which currently seems to be the only to "pass through" args via the argh crate, as otherwise it attempts
        // to interpret them.
        let mut command = start.command.clone();
        let mut args: Option<Vec<String>> = None;

        // see if there's a space in the command...
        if let Some(pos) = command.find(' ') {
            // create temporary, so we can split_at()
            let temp_command = command.clone();
            // if so assume there are "bundled" sequential args we want to extract after the first space
            let (split_command, remainder) = temp_command.split_at(pos);
            command = split_command.to_string();
            let remainder = remainder.trim_start();
            if !remainder.is_empty() {
                args = Some(remainder.split(' ').map(str::to_string).collect());
            }

            // if there are further args in the command line args, append them afterwards
            if !start.args.is_empty() {
                if let Some(ref mut local_args) = &mut args {
                    local_args.append(&mut start.args.clone());
                }
                else {
                    args = Some(start.args.clone());
                }
            }
        }

        eprintln!("Starting process: {}", command);

        let recorder: Option<ProcessRecorderRun> = ProcessRecorderRun::new(&command, args, &record_params);
        if recorder.is_none() {
            // Note: this isn't actually that useful, as the process isn't actually started until start() is called...
            eprintln!("Error starting process...");
            return;
        }

        let mut recorder: ProcessRecorderRun = recorder.unwrap();
        // Note: start() prints some progress...
        if !recorder.start(has_been_cancelled_flag) {
            // failed to start process, so return out
            return;
        }

        recording_results = Some(recorder.get_recording());

        eprintln!("Recorded process has exited.");
    }

    if let Some(export_path) = args.export {
        if let Some(rec_results) = recording_results {
            // save the results

            rec_results.save_to_csv_file(&export_path, true);

            eprintln!("Saved results to file: {}", export_path);
        }
    }
}
