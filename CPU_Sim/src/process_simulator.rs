use std::collections::LinkedList;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use crate::{MAX_PROCESSES, KILL, TERMINATED};
use crate::process::Process;

pub fn process_simulator_threads (handles: &mut Vec<thread::JoinHandle<()>>, num_threads : i32, ready_queue : &Arc<Mutex<LinkedList<Process>>>, ready_queue_condvar : &Arc<Condvar>, terminated_queue : &Arc<Mutex<LinkedList<Process>>>, terminated_queue_condvar : &Arc<Condvar>) {

    // while not all processes terminated, pop process from ready queue, run it
    // add back to queue if not terminated
    // if terminated discard

    let total_terminated = Arc::new(Mutex::new(0));

    for i in 0..num_threads as usize {

        let ready_queue = Arc::clone(ready_queue);
        let ready_queue_condvar = Arc::clone(ready_queue_condvar);

        let terminated_queue = Arc::clone(terminated_queue);
        let terminated_queue_condvar = Arc::clone(terminated_queue_condvar);

        let total_terminated = Arc::clone(&total_terminated);

        let handle = thread::spawn(move || {

            println!("SIMULATOR {} CREATED", i);

            loop {

                // thread should wait while linked list isn't empty
                let mut ready_guard = ready_queue.lock().unwrap();

                while ready_guard.is_empty() {
                    ready_guard = ready_queue_condvar.wait(ready_guard).unwrap();
                }

                let mut process = ready_guard.pop_front().unwrap();
                println!("READY QUEUE REMOVED, PID: {}", process.pid);

                // if process is kill process
                if process.pid == -1 && process.state == KILL {

                    // release resource and give up reserved space
                    drop(ready_guard);
                    ready_queue_condvar.notify_all();
                    break;
                }

                process.run();
                println!("PROCESS RUN, PID: {}, REMAINING TIME: {}", process.pid, process.remaining_burst_time);

                if process.state == TERMINATED {

                    // release resource and give up reserved space
                    ready_queue_condvar.notify_all();
                    drop(ready_guard);

                    // lock total terminated and increment
                    let mut total_terminated_num = total_terminated.lock().unwrap();
                    *total_terminated_num += 1;

                    // thread waits until space in terminated queue
                    let mut terminated_guard = terminated_queue.lock().unwrap();
                    while terminated_guard.len() == MAX_PROCESSES as usize {
                        terminated_guard = terminated_queue_condvar.wait(terminated_guard).unwrap();
                    }

                    // once done waiting, add process to terminated queue
                    println!("TERMINATED QUEUE ADD, PID: {}", process.pid);
                    terminated_guard.push_back(process);
                    drop(terminated_guard);
                    terminated_queue_condvar.notify_all();
                } else {
                    // space is reserved in ready queue
                    println!("READY QUEUE ADD, PID: {}", process.pid);
                    ready_guard.push_back(process);
                    drop(ready_guard);
                    ready_queue_condvar.notify_all();
                }
            }

            println!("SIMULATOR {} FINISHED", i);
        });

        handles.push(handle);
    }
}