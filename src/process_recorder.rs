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

    pub normalise_cpu_usage:    bool,

    pub print_values:           bool,
}

impl ProcessRecordParams {
    pub fn new() -> ProcessRecordParams {
        ProcessRecordParams { sample_interval: 1, record_duration: None,
                             normalise_cpu_usage: true, print_values: false }
    }

    pub fn set_sample_interval(&mut self, interval_secs: u64) {
        self.sample_interval = interval_secs;
    }

    pub fn set_normalise_cpu_usage(&mut self, normalise_cpu_usage: bool) {
        self.normalise_cpu_usage = normalise_cpu_usage;
    }

    pub fn set_print_values(&mut self, print_values: bool) {
        self.print_values = print_values;
    }
}

pub trait ProcessRecorder {
    fn start(&mut self) -> bool;

    fn get_recording(&self) -> ProcessRecording;
}

pub struct ProcessRecorderCore {
    recorder_params:    ProcessRecordParams,
    process:            Option<Process>,

    print_values:       bool,

    pub recording:      ProcessRecording,

    pub start_time:     Option<SystemTime>,
}

impl ProcessRecorderCore {
    // pub fn new() -> ProcessRecorderCore {
    //     ProcessRecorderCore { recorder_params: ProcessRecordParams::new(),
    //                           process: None,
    //                           print_values: false,
    //                           recording: ProcessRecording::new(0), start_time: None }
    // }

    pub fn from_params(params: &ProcessRecordParams) -> ProcessRecorderCore {
        ProcessRecorderCore { recorder_params: params.clone(),
                              process: None,
                              print_values: params.print_values,
                              recording: ProcessRecording::new(params, 0), start_time: None }
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
        let mut cpu_usage_perc = process.cpu_percent().unwrap_or(0.0);
        // so because of that, normalise the value to the number of threads if requested (which is the default).
        if self.recorder_params.normalise_cpu_usage {
            cpu_usage_perc /= self.recording.num_system_threads as f32;
        }
        if let Ok(mem) = process.memory_info() {

            let elapsed_time = self.start_time.unwrap().elapsed().unwrap().as_secs_f32();

            let new_sample = Sample { elapsed_time, cpu_usage: cpu_usage_perc, curr_rss: mem.rss() };

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

        let mut core = ProcessRecorderCore::from_params(record_params);
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

    // TODO: get rid of the need to do this with a copy...
    fn get_recording(&self) -> ProcessRecording {
        return self.core.recording.clone();
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
                                        core: ProcessRecorderCore::from_params(record_params), child_process: None })
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

    // TODO: get rid of the need to do this with a copy...
    fn get_recording(&self) -> ProcessRecording {
        return self.core.recording.clone();
    }
}
