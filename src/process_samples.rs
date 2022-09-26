

pub struct ProcessRecording {

    // start timestamp

    pub initial_process_id:     u32,
    pub current_process_id:     u32,

    pub samples:        Vec<Sample>,
}

impl ProcessRecording {
    pub fn new(initial_process_id: u32) -> ProcessRecording {
        ProcessRecording { initial_process_id, current_process_id: initial_process_id, samples: Vec::with_capacity(128) }
    }
}

pub struct Sample {
    // in seconds
    pub elapsed_time:       f32,

    // 100.0 is full cpu usage (all cores)
    pub cpu_usage:          f32,

    // in bytes
    pub curr_rss:           u64,
    pub peak_rss:           u64,
}
