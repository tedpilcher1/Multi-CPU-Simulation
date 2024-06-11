use std::collections::LinkedList;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use crate::{MAX_CONCURRENT_PROCESSES, MAX_PROCESSES};
use crate::process::Process;

pub fn process_terminator_threads (handles: &mut Vec<thread::JoinHandle<()>>, num_threads : i32, num_sim_threads : i32, terminated_queue : &Arc<Mutex<LinkedList<Process>>>, terminated_queue_condvar : &Arc<Condvar>, pid_pool : &Arc<Mutex<LinkedList<i32>>>, pid_pool_condvar :  &Arc<Condvar>, ready_queue : &Arc<Mutex<LinkedList<Process>>>, ready_queue_condvar : &Arc<Condvar>) {

    let total_dropped = Arc::new(Mutex::new(0));
    let is_first_done = Arc::new(Mutex::new(false));

    for i in 0..num_threads as usize {

        let terminated_queue = Arc::clone(terminated_queue);
        let terminated_queue_condvar = Arc::clone(terminated_queue_condvar);

        let pid_pool = Arc::clone(pid_pool);
        let pid_pool_condvar = Arc::clone(pid_pool_condvar);

        let ready_queue = Arc::clone(ready_queue);
        let ready_queue_condvar = Arc::clone(ready_queue_condvar);

        let total_dropped = Arc::clone(&total_dropped);
        let is_first_done = Arc::clone(&is_first_done);

        let handle = thread::spawn(move || {

            println!("DAEMON THREAD {} CREATED", i);

            while *total_dropped.lock().unwrap() < MAX_PROCESSES {

                // lock total terminated and increment
                let mut total_dropped_num = total_dropped.lock().unwrap();
                *total_dropped_num += 1;

                // release mutex guard
                drop(total_dropped_num);

                // thread should wait until terminated queue has process
                let mut terminated_guard = terminated_queue.lock().unwrap();

                while terminated_guard.is_empty() {

                    terminated_guard = terminated_queue_condvar.wait(terminated_guard).unwrap();
                }

                // get process from terminated queue
                let process: Process = terminated_guard.pop_front().unwrap();
                println!("DAEMON THREAD: {}, TERMINATED QUEUE REMOVED, PID: {}", i, process.pid);


                // release guard
                drop(terminated_guard);
                terminated_queue_condvar.notify_all();

                // wait until pid pool has space and release pid back into pool
                let mut pid_guard = pid_pool.lock().unwrap();
                while pid_guard.len() == MAX_CONCURRENT_PROCESSES as usize {

                    pid_guard = pid_pool_condvar.wait(pid_guard).unwrap();
                }

                pid_guard.push_back(process.pid);

                // release pid guard
                pid_pool_condvar.notify_all();
                drop(pid_guard);

                // drop process
                println!("DAEMON THREAD: {}, PROCESS TERMINATED: PID {}", i, process.pid);
                drop(process);
            }

            // if first to finish, add num_simulators termination processes to ready queue
            // each tells a simulator to terminate
            let mut is_first_done_bool = is_first_done.lock().unwrap();

            if !*is_first_done_bool {

                *is_first_done_bool = true;

                drop(is_first_done_bool);

                // wait for space in ready queue
                // should be guaranteed as all processed terminated
                // but better to be safe i guess
                // thread should wait while ready queue is full
                let mut ready_guard = ready_queue.lock().unwrap();
                while !ready_guard.len() == MAX_CONCURRENT_PROCESSES as usize {

                    ready_guard = ready_queue_condvar.wait(ready_guard).unwrap();
                }

                for i in 0..num_sim_threads {
                    ready_guard.push_back(Process::generate_sim_terminate_process());
                    println!("DAEMON THREAD: {}, KILL PROCESS ADDED TO READY QUEUE", i);
                }

                drop(ready_guard);
                ready_queue_condvar.notify_all();
            }

            println!("DAEMON THREAD {} FINISHED", i);
        });

        handles.push(handle);
    }
}