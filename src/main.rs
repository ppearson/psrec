
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

    #[argh(option)]
    /// whether to output peak RSS values (a high-watermark of memory usage)
    peak_rss: Option<bool>,
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
    #[argh(option)]
    /// PID of process to attach to and record
    pid: i32,
}


fn main() {
    let args: MainArgs = argh::from_env();

    if let SubCommandEnum::Start(start) = args.command {
        eprintln!("Starting process: {}", start.command);
    
        let params = ProcessRecordParams::new();

        let mut recorder: Option<ProcessRecorderRun> = ProcessRecorderRun::new(&start.command, Some(start.args.clone()), &params);
        if recorder.is_none() {
            return;
        }
        let mut recorder: ProcessRecorderRun = recorder.unwrap();
        recorder.start();

    }


    println!("Hello, world!");
}
