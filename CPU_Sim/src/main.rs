use std::collections::LinkedList;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use crate::process::Process;
use crate::process_generator::process_generator_threads;
use crate::process_simulator::process_simulator_threads;
use crate::process_terminator::process_terminator_threads;

mod process;
mod process_generator;
mod process_simulator;
mod process_terminator;

static READY: i32 = 0;
static RUNNING: i32 = 1;
static TERMINATED: i32 = 2;
static KILL: i32 = -1;
static MAX_PROCESSES: i32 = 12;
static MAX_CONCURRENT_PROCESSES: i32 = 5;

fn setup_pid_pool(pid_pool : &Arc<Mutex<LinkedList<i32>>>){

    let mut pid_pool_queue = pid_pool.lock().unwrap();

    for i in 0..MAX_CONCURRENT_PROCESSES {

        pid_pool_queue.push_back(i);
    }
}

fn main() {

    // thread nums
    let num_generators = 1;
    let num_simulators = 2;
    let num_terminators = 1;

    // queues
    // ready queue
    let ready_queue = Arc::new(Mutex::new(LinkedList::new()));
    let ready_queue_condvar = Arc::new(Condvar::new());

    // terminated queue
    let terminated_queue = Arc::new(Mutex::new(LinkedList::new()));
    let terminated_queue_condvar = Arc::new(Condvar::new());

    // pid pool (queue)
    let pid_pool = Arc::new(Mutex::new(LinkedList::new()));
    let pid_pool_condvar = Arc::new(Condvar::new());

    setup_pid_pool(&pid_pool);

    let mut handles = Vec::new();

    // create process generator threads
    process_generator_threads(&mut handles, num_generators, &ready_queue, &ready_queue_condvar, &pid_pool, &pid_pool_condvar);

    // create process simulator threads
    process_simulator_threads(&mut handles, num_simulators, &ready_queue, &ready_queue_condvar, &terminated_queue, &terminated_queue_condvar);

    // create process terminator threads
    process_terminator_threads(&mut handles, num_terminators, num_simulators,&terminated_queue, &terminated_queue_condvar, &pid_pool, &pid_pool_condvar, &ready_queue, &ready_queue_condvar);

    for handle in handles {

        handle.join().unwrap();
    }
}
