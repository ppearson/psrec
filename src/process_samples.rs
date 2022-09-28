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

use chrono::{Local, DateTime};

use crate::process_recorder::ProcessRecordParams;

pub struct ProcessRecording {

    recorder_params:    ProcessRecordParams,

    // start timestamp
    pub start_timestamp:        DateTime<Local>,

    pub initial_process_id:     u32,
    pub current_process_id:     u32,

    // used for normalising CPU usage if needed... (depending on configuration)
    pub num_system_threads:     u32,

    pub samples:        Vec<Sample>,
}

impl ProcessRecording {
    pub fn new(recorder_params: &ProcessRecordParams, initial_process_id: u32) -> ProcessRecording {

        let mut num_threads = 1u32;
        if let Ok(nt) = std::thread::available_parallelism() {
            num_threads = nt.get() as u32;
        }
        ProcessRecording { recorder_params: recorder_params.clone(),
                           start_timestamp: Local::now(),
                           initial_process_id,
                           current_process_id: initial_process_id,
                           num_system_threads: num_threads,
                           samples: Vec::with_capacity(128) }
    }

    pub fn set_start_time(&mut self) {
        self.start_timestamp = Local::now();
    }
}

pub struct Sample {
    // in seconds
    pub elapsed_time:       f32,

    // Note: this value may or may not be normalised (to 100.0 if so), depending on the params
    pub cpu_usage:          f32,

    // in bytes
    pub curr_rss:           u64,

//    pub peak_rss:           u64,
}
