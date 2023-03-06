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

use crate::process_sampler::{ProcessSampler};
use crate::process_samples::Sample;
use crate::process_recorder::ProcessRecordParams;

use procfs::process::{Process};
use std::time::Instant;

// Note: this is the "advanced" sampler that only works on Linux, and uses the /proc/<pid> file system
//       to extract more advanced info.

pub struct ProcessSamplerAdvanced {
    recorder_params: ProcessRecordParams,
    pid: u32,
    process: Process,

    // cached stuff 

    // ticks per second
    tps:        u64,

    page_size:  u64,

    // previous values
    last_utime:      u64,
    last_stime:      u64,
    last_cutime:     u64,
    last_cstime:     u64,

    last_time_instant:  Instant,
}

impl ProcessSamplerAdvanced {
    pub fn new(recorder_params: ProcessRecordParams, pid: u32) -> Option<ProcessSamplerAdvanced> {
        let process = Process::new(pid as i32);
        if let Err(err) = process {
            eprintln!("Error accessing process pid: {}, {}", pid, err.to_string());
            return None;
        }

        let tps = procfs::ticks_per_second();
        let page_size = procfs::page_size();

        let process = process.unwrap();

        // take a snapshot of the cpu time values, so that the first time get_sample() is called it should be valid with an average
        // since now...
        
        let stat = &process.stat();
        if let Err(err) = stat {
            eprintln!("Error getting stat info from process with pid: {}, {}", pid, err.to_string());
            return None;
        }

        let instant = Instant::now();
        let stat = stat.as_ref().unwrap();
    
        Some(ProcessSamplerAdvanced { recorder_params, pid,
                                    process: process,
                                    tps,
                                    page_size,
                                    last_utime: stat.utime,
                                    last_stime: stat.stime,
                                    last_cutime: stat.cutime as u64,
                                    last_cstime: stat.cstime as u64,
                                    last_time_instant: instant
                                       })
    }
}

impl ProcessSampler for ProcessSamplerAdvanced {
    fn get_sample(&mut self) -> Option<Sample> {
        let stat = &self.process.stat();
        if let Err(err) = stat {
            eprintln!("Error getting stat info from process within get_sample() call: {}", err.to_string());
            return None;
        }

        let stat = stat.as_ref().unwrap();
        let instant = Instant::now();

        let elapsed = instant.duration_since(self.last_time_instant).as_secs_f64();
        if elapsed == 0.0 {
            // return None for the moment, so 
            return None;
        }

        // let kern_stat = procfs::KernelStats::new();
        // if let Err(err) = kern_stat {
        //     eprintln!("Error getting stat info from kernel within get_sample() call: {}", err.to_string());
        //     return None;
        // }

        // let kern_stat = kern_stat.unwrap();

        // calc total possible
        // TODO: do we also want iowait, idle, irq, softirq as well?
 //       let total_possible_time = kern_stat.total.user + kern_stat.total.system + kern_stat.total.idle + kern_stat.total.nice;

        let last_full_time_count = (self.last_utime + self.last_stime + self.last_cutime + self.last_cstime) as f64;

        let this_full_time_count = (stat.utime + stat.stime + stat.cutime as u64 + stat.cstime as u64) as f64;

        // let cpu_usage =  ((this_full_time_count - last_full_time_count) * 1000 / self.tps ) as f64 / elapsed;
        // let cpu_usage =  ((this_full_time_count - last_full_time_count) * 1000 / self.tps ) as f64;

        // this gives us absolute CPU usage, i.e. one full thread is 100.0, four threads is 400.0, etc.
        let cpu_usage = 100.0 * (((this_full_time_count - last_full_time_count) / self.tps as f64) / elapsed);

        let full_rss = stat.rss * self.page_size;

        // replace cached values
        self.last_time_instant = instant;
        self.last_utime = stat.utime;
        self.last_stime = stat.stime;
        self.last_cutime = stat.cutime as u64;
        self.last_cstime = stat.cstime as u64;

        // set 0.0 as the time, it will be replaced later...
        let new_sample = Sample { elapsed_time: 0.0, cpu_usage: cpu_usage as f32, curr_rss: full_rss };
        return Some(new_sample);
    }
}