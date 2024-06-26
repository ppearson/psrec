/*
 psrec
 Copyright 2022-2024 Peter Pearson.
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

use crate::process_samples::Sample;

use psutil::process::Process;

pub trait ProcessSampler {
    fn get_sample(&mut self) -> Option<Sample> {
        None
    }
}

pub struct ProcessSamplerBasic {
    process: Process,
}

impl ProcessSamplerBasic {
    pub fn new(pid: u32) -> Option<ProcessSamplerBasic> {
        let process = Process::new(pid);
        if let Err(err) = process {
            eprintln!("Error attaching to PID: {}, {}", pid, err);
            return None;
        }

        Some(ProcessSamplerBasic { process: process.unwrap() })
    }
}

impl ProcessSampler for ProcessSamplerBasic {
    fn get_sample(&mut self) -> Option<Sample> {
        let process = &mut self.process;

        let cpu_usage_perc = process.cpu_percent().unwrap_or(0.0);

        // Note: num_threads() is not implemented yet in psutil...
        
        if let Ok(mem) = process.memory_info() {
            // set 0.0 as the time, it will be replaced later...
            let new_sample = Sample { elapsed_time: 0.0, cpu_usage: cpu_usage_perc, curr_rss: mem.rss(), thread_count: 0 };
            return Some(new_sample);
        }

        None
    }
}
