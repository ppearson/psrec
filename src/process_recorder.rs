
use crate::process_samples::*;

use std::process::Command;
use std::time::SystemTime;

use std::thread::sleep;

use psutil::process::Process;

#[derive(Clone, Debug)]
pub struct ProcessRecordParams {
    // in seconds
    pub sample_interval:        u64,

    // in seconds
    pub record_duration:        Option<usize>,
}

impl ProcessRecordParams {
    pub fn new() -> ProcessRecordParams {
        ProcessRecordParams { sample_interval: 1, record_duration: None }
    }

    pub fn set_sample_interval(mut self, interval_secs: u64) -> Self {
        self.sample_interval = interval_secs;
        return self;
    }
}

pub trait ProcessRecorder {
    fn start(&mut self) -> bool {
        return false;
    }

}

pub struct ProcessRecorderCore {
    process:        Option<Process>,

    pub recording:  ProcessRecording,

    pub start_time: Option<SystemTime>,
}

impl ProcessRecorderCore {
    pub fn new() -> ProcessRecorderCore {
        ProcessRecorderCore { process: None, recording: ProcessRecording::new(0), start_time: None }
    }

    fn process_is_running(&mut self) -> bool {
        if self.process.is_none() {
            return false;
        }

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

        let cpu_usage_perc = process.cpu_percent().unwrap_or(0.0);
        if let Ok(mem) = process.memory_info() {

            let elapsed_time = self.start_time.unwrap().elapsed().unwrap().as_secs_f32();

            let new_sample = Sample { elapsed_time, cpu_usage: cpu_usage_perc, curr_rss: mem.rss(), peak_rss: 0 };

            eprintln!("Time: {:.2}\tCPU: {:.1}%\t\tMem: {} KB", elapsed_time, cpu_usage_perc, new_sample.curr_rss / 1024);

            self.recording.samples.push(new_sample);
        }
    }
}

pub struct ProcessRecorderRun {
    record_params:  ProcessRecordParams,

    command:        String,

    args:           Option<Vec<String>>,

    core:           ProcessRecorderCore,
}

impl ProcessRecorderRun {
    pub fn new(command: &str, args: Option<Vec<String>>, record_params: &ProcessRecordParams) -> Option<ProcessRecorderRun> {
        if command.is_empty() {
            return None;
        }

        return Some(ProcessRecorderRun { record_params: record_params.clone(), command: command.to_string(), args,
                                        core: ProcessRecorderCore::new() })
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
            if let Err(err) = process {
                eprintln!("Error monitoring process.");
                return false;
            }

            self.core.start_time = Some(SystemTime::now());

            self.core.process = Some(process.unwrap());

            self.core.record_sample();

            let sleep_duration = std::time::Duration::from_secs(self.record_params.sample_interval);

            while self.core.process_is_running() {
                std::thread::sleep(sleep_duration);

                self.core.record_sample();
            }
            
            return true;
        }

        return false;
    }

    
}
