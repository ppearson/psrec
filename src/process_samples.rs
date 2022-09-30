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

use std::{fs::File, io::BufWriter, io::Write};

use chrono::{Local, DateTime};

use crate::process_recorder::ProcessRecordParams;

#[derive(Clone, Debug)]
pub struct Sample {
    // in seconds
    pub elapsed_time:       f32,

    // Note: this value may or may not be normalised (to 100.0 if so), depending on the params
    pub cpu_usage:          f32,

    // in bytes
    pub curr_rss:           u64,

//    pub peak_rss:           u64,
}

#[derive(Clone, Debug)]
pub struct ProcessRecording {

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
        // TODO: is this always going to be correct?
        if let Ok(nt) = std::thread::available_parallelism() {
            num_threads = nt.get() as u32;
        }
        ProcessRecording { start_timestamp: Local::now(),
                           initial_process_id,
                           current_process_id: initial_process_id,
                           num_system_threads: num_threads,
                           samples: Vec::with_capacity(128) }
    }

    pub fn set_start_time(&mut self) {
        self.start_timestamp = Local::now();
    }

    // TODO: use Result properly for return code...
    pub fn save_to_csv_file(&self, output_file_path: &str, add_metadata_comments: bool) -> bool {
        let file = File::create(output_file_path);
        if file.is_err() {
            eprintln!("Error saving results to CSV file: {}", output_file_path);
            return false;
        }
        let mut buf_writer = BufWriter::new(file.unwrap());

        if add_metadata_comments {
            writeln!(buf_writer, "# Process recording.").unwrap();

            writeln!(buf_writer, "# Time elapsed,CPU Usage,RSS").unwrap();
        }

        for sample in &self.samples {
            writeln!(buf_writer, "{:.1},{:.1},{}", sample.elapsed_time, sample.cpu_usage, sample.curr_rss).unwrap();
        }

        return true;
    }
}
