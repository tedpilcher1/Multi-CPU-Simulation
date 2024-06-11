use std::{thread, time::Duration};
static TIME_SLICE: u64 = 25;
static READY: i32 = 0;
static RUNNING: i32 = 1;
static TERMINATED: i32 = 2;
static KILL: i32 = -1;
static MAX_BUST: f64 = 100.0;

pub struct Process {

    pub pid: i32,
    pub burst_time: f64,
    pub remaining_burst_time: f64,
    pub state: i32,
}


impl Process {

    pub fn generate_process(pid: i32) -> Process {

        let process = Process {
            pid,
            burst_time: MAX_BUST,
            remaining_burst_time: MAX_BUST,
            state: READY,
        };

        return process;
    }

    pub fn generate_sim_terminate_process() -> Process {

        let process = Process {
            pid: -1,
            burst_time: MAX_BUST,
            remaining_burst_time: MAX_BUST,
            state: KILL,
        };

        return process;
    }

    pub fn run(&mut self) {

        self.state = RUNNING;

        if TIME_SLICE < self.remaining_burst_time as u64 {

            thread::sleep(Duration::from_millis(TIME_SLICE));
            self.remaining_burst_time -= TIME_SLICE as f64;
        }

        else {

            thread::sleep(Duration::from_millis(self.remaining_burst_time as u64));
            self.remaining_burst_time = 0.0;
        }

        // check if process should be terminated
        if self.remaining_burst_time == 0.0 {

            self.state = TERMINATED;
        }

        else {

            self.state = READY;
        }
    }
}