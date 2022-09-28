/*
 psrec
 Copyright 2022 Peter Pearson.
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

mod process_samples;
mod process_recorder;

use argh::FromArgs;

use crate::process_recorder::*;

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct MainArgs {
    #[argh(subcommand)]
    command: SubCommandEnum,

    #[argh(option, short = 'i')]
    /// interval between each sample of the process. Default is 2 seconds.
    interval: Option<usize>,

    #[argh(option, short = 'd')]
    /// duration to record for in seconds. By default will be until the process being recorded ends.
    duration: Option<usize>,

    /// whether to use absolute (clock) times instead of time elapsed after start
    #[argh(switch, short = 'a')]
    absolute_timestamps: bool,

    /// whether to record cpu usage as Absolute quantities, instead of Normalised quantities (default).
    /// Absolute will scale over 100.0 for the number of threads, so 800.0 will be 8 threads using full CPU.
    /// Normalised will be normalised to 100.0, so instead of 800.0 in the above example, it will be 100.0.
    #[argh(switch)]
    absolute_cpu_usage: bool,

    /// whether to print out values live as process is being recorded to stderr
    #[argh(switch)]
    print_values: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommandEnum {
    Start(SubCommandStart),
    Attach(SubCommandAttach),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Start a process with command line args.
#[argh(subcommand, name = "start")]
struct SubCommandStart {
    #[argh(positional)]
    /// command line command to start/run and record
    command: String,

    #[argh(positional)]
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

    let mut record_params = ProcessRecordParams::new();

    // TODO: something better than this... pass in args to ProcessRecordParams?
    //       or at least encapsulate it somewhere...
    if let Some(interval) = args.interval {
        record_params.set_sample_interval(interval as u64);
    }
    if args.print_values {
        record_params.set_print_values(true);
    }
    if args.absolute_cpu_usage {
        record_params.set_normalise_cpu_usage(false);
    }

    if let SubCommandEnum::Attach(attach) = args.command {
        eprintln!("Attaching to process PID: {}...", attach.pid);

        let recorder: Option<ProcessRecorderAttach> = ProcessRecorderAttach::new(attach.pid, &record_params);

        if recorder.is_none() {
            eprintln!("Error attaching to process...");
            return;
        }
        let mut recorder: ProcessRecorderAttach = recorder.unwrap();
        recorder.start();

        eprintln!("Attached process has exited.");
    }
    else if let SubCommandEnum::Start(start) = args.command {
        eprintln!("Starting process: {}", start.command);

        let recorder: Option<ProcessRecorderRun> = ProcessRecorderRun::new(&start.command, Some(start.args.clone()), &record_params);
        if recorder.is_none() {
            eprintln!("Error starting process...");
            return;
        }
        let mut recorder: ProcessRecorderRun = recorder.unwrap();
        recorder.start();

        eprintln!("Recorded process has exited.");
    }
}
