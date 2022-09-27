
use crate::process_samples::*;

use std::process::Command;
use std::time::SystemTime;

use psutil::process::Process;

#[derive(Clone, Debug)]
pub struct ProcessRecordParams {
    // in seconds
    pub sample_interval:        u64,

    // in seconds
    pub record_duration:        Option<usize>,

    pub print_values:           bool,
}

impl ProcessRecordParams {
    pub fn new() -> ProcessRecordParams {
        ProcessRecordParams { sample_interval: 2, record_duration: None, print_values: false }
    }

    pub fn set_sample_interval(&mut self, interval_secs: u64) {
        self.sample_interval = interval_secs;
    }

    pub fn set_print_values(&mut self, print_values: bool) {
        self.print_values = print_values;
    }
}

pub trait ProcessRecorder {
    fn start(&mut self) -> bool;
}

pub struct ProcessRecorderCore {
    process:        Option<Process>,

    print_values:   bool,

    pub recording:  ProcessRecording,

    pub start_time: Option<SystemTime>,
}

impl ProcessRecorderCore {
    pub fn new() -> ProcessRecorderCore {
        ProcessRecorderCore { process: None, print_values: false, recording: ProcessRecording::new(0), start_time: None }
    }

    pub fn from_params(params: &ProcessRecordParams) -> ProcessRecorderCore {
        ProcessRecorderCore { process: None, print_values: params.print_values, recording: ProcessRecording::new(0), start_time: None }
    }

    // Note: this apparently can't be relied on for processes we fork/spawn ourselves...
    fn process_is_running(&mut self) -> bool {
        if self.process.is_none() {
            return false;
        }

        // Note: calling psutil::process::Process::is_running() on a process we spawned ourself
        // is apparently not useful, as it still returns true even when the process has actually exited.
        if self.process.as_ref().unwrap().is_running() {
            return true;
        }

        return false;
    }

    fn record_sample(&mut self) {
        if self.process.is_none() {
            return;
        }

        let process = self.process.as_mut().unwrap();

        // Note: cpu_percent() is per-core, so 100.0 is 1 full CPU core, 800.0 is 8 full threads being used.
        let cpu_usage_perc = process.cpu_percent().unwrap_or(0.0);
        if let Ok(mem) = process.memory_info() {

            let elapsed_time = self.start_time.unwrap().elapsed().unwrap().as_secs_f32();

            let new_sample = Sample { elapsed_time, cpu_usage: cpu_usage_perc, curr_rss: mem.rss(), peak_rss: 0 };

            if self.print_values {
                eprintln!("Time: {:.2}\tCPU: {:.1}%\t\tMem: {} KB", elapsed_time, cpu_usage_perc, new_sample.curr_rss / 1024);
            }

            self.recording.samples.push(new_sample);
        }
    }
}

pub struct ProcessRecorderAttach {
    record_params:  ProcessRecordParams,

    core:           ProcessRecorderCore,
}

impl ProcessRecorderAttach {
    pub fn new(pid: u32, record_params: &ProcessRecordParams) -> Option<ProcessRecorderAttach> {
        let process = Process::new(pid);
        if let Err(_err) = process {
            eprintln!("Error monitoring process.");
            return None;
        }

        let mut core = ProcessRecorderCore::from_params(&record_params);
        core.process = Some(process.unwrap());

        return Some(ProcessRecorderAttach { record_params: record_params.clone(), core })
    }
}

impl ProcessRecorder for ProcessRecorderAttach {
    fn start(&mut self) -> bool {
        self.core.start_time = Some(SystemTime::now());

        self.core.record_sample();

        let sleep_duration = std::time::Duration::from_secs(self.record_params.sample_interval);

        std::thread::sleep(sleep_duration);

        while self.core.process_is_running() {
            self.core.record_sample();

            // TODO: this suffers from a tiny bit of drift...
            std::thread::sleep(sleep_duration);
        }
        
        return true;
    }
}

pub struct ProcessRecorderRun {
    record_params:  ProcessRecordParams,

    command:        String,
    args:           Option<Vec<String>>,

    core:           ProcessRecorderCore,

    // actual spawned/forked process ownership
    child_process:  Option<std::process::Child>,
}

impl ProcessRecorderRun {
    pub fn new(command: &str, args: Option<Vec<String>>, record_params: &ProcessRecordParams) -> Option<ProcessRecorderRun> {
        if command.is_empty() {
            return None;
        }

        return Some(ProcessRecorderRun { record_params: record_params.clone(), command: command.to_string(), args,
                                        core: ProcessRecorderCore::from_params(&record_params), child_process: None })
    }

    // this is needed because we can't rely on psutil::process::Process::is_running() working
    // on a forked/spawned process we started ourselves apparently.
    fn check_process_is_running(&mut self) -> bool {
        if self.child_process.is_none() {
            return false;
        }

        // check if the process has exited with an exit code.
        // This does not block, so we can call it as a poll-like event loop
        let still_running = match self.child_process.as_mut().unwrap().try_wait() {
            Ok(Some(_status)) =>    false,
            Ok(None) =>             true, // still running
            Err(_err) =>            false, // not quite correct, but for the moment...
        };
        return still_running;
    }
}

impl ProcessRecorder for ProcessRecorderRun {
    fn start(&mut self) -> bool {

        // spawn a forked process to run the process we're going to monitor in...

        let mut command = Command::new(&self.command);
        if let Some(args) = &self.args {
            if !args.is_empty() {
                command.args(args);
            }
        }

        let spawn_res = command.spawn();

        if let Ok(child_info) = spawn_res {
            let process = Process::new(child_info.id());
            if let Err(_err) = process {
                eprintln!("Error monitoring process.");
                return false;
            }
            self.child_process = Some(child_info);

            self.core.start_time = Some(SystemTime::now());

            self.core.process = Some(process.unwrap());

            self.core.record_sample();

            let sleep_duration = std::time::Duration::from_secs(self.record_params.sample_interval);

            std::thread::sleep(sleep_duration);

            while self.check_process_is_running() {
                self.core.record_sample();

                // TODO: this suffers from a tiny bit of drift...
                std::thread::sleep(sleep_duration);
            }
            
            return true;
        }

        return false;
    }

    
}
