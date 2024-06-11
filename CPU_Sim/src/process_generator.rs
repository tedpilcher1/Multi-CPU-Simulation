use std::collections::LinkedList;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use crate::{MAX_CONCURRENT_PROCESSES, MAX_PROCESSES};
use crate::process::Process;


pub fn process_generator_threads(handles: &mut Vec<thread::JoinHandle<()>>, num_threads : i32, ready_queue : &Arc<Mutex<LinkedList<Process>>>, ready_queue_condvar : &Arc<Condvar>, pid_pool : &Arc<Mutex<LinkedList<i32>>>, pid_pool_condvar :  &Arc<Condvar>) {

    // while there are less than MAX_CONCURRENT_PROCESSES add new process to system
    // until NUM_PROCESS reached

    let total_gen = Arc::new(Mutex::new(0));

    for i in 0..num_threads{

        let ready_queue = Arc::clone(ready_queue);
        let ready_queue_condvar = Arc::clone(ready_queue_condvar);

        let pid_pool = Arc::clone(pid_pool);
        let pid_pool_condvar = Arc::clone(pid_pool_condvar);

        let total_gen = Arc::clone(&total_gen);


        let handle = thread::spawn(move || {

            println!("GENERATOR {} CREATED", i);

            while *total_gen.lock().unwrap() < MAX_PROCESSES {

                // lock total added and increment
                let mut total_gen_num = total_gen.lock().unwrap();
                *total_gen_num += 1;

                // release mutex guard
                drop(total_gen_num);

                // get pid from pool, should wait until pid pool isn't empty
                let mut pid_guard = pid_pool.lock().unwrap();
                while pid_guard.is_empty() {

                    pid_guard = pid_pool_condvar.wait(pid_guard).unwrap();
                }

                let pid: i32 = pid_guard.pop_front().unwrap();

                // release pid pool
                drop(pid_guard);
                pid_pool_condvar.notify_all();

                // thread should wait while ready queue is full
                let mut ready_guard = ready_queue.lock().unwrap();
                while !ready_guard.len() == MAX_CONCURRENT_PROCESSES as usize {

                    ready_guard = ready_queue_condvar.wait(ready_guard).unwrap();
                }

                // could release guard here and get again after generating
                // might make it faster, might not
                // for simplicity I won't

                // create process and add to ready queue
                let process = Process::generate_process(pid);
                println!("PROCESS GENERATED, PID: {}, GEN THREAD: {}", process.pid, i);
                println!("READY QUEUE ADD, PID: {}", process.pid, );
                ready_guard.push_back(process);
            }

            println!("GENERATOR {} FINISHED", i);
        });

        handles.push(handle);
    }
}