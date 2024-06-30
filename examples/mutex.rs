use std::sync::atomic::{AtomicU8, Ordering};
use std::thread;

use clap::Parser;
use raw_sync::locks::*;
use shared_memory::*;

/// Spawns N threads that increment a value to 10 using a mutex
#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Number of threads to spawn
    num_threads: usize,

    /// Count to this value
    #[clap(long, short, default_value_t = 50)]
    count_to: u8,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    if args.num_threads < 1 {
        eprintln!("num_threads should be 2 or more");
        return;
    }
    let mut threads = Vec::with_capacity(args.num_threads);
    let _ = std::fs::remove_file("mutex_mapping");

    // Spawn N threads
    for i in 0..args.num_threads {
        let thread_id = i + 1;
        threads.push(thread::spawn(move || {
            increment_value("mutex_mapping", thread_id);
        }));
    }

    // Wait for threads to exit
    for t in threads.drain(..) {
        t.join().unwrap();
    }
}

fn increment_value(shmem_flink: &str, thread_num: usize) {
    // Create or open the shared memory mapping
    let shmem = match ShmemConf::new().size(4096).flink(shmem_flink).create() {
        Ok(m) => m,
        Err(Error::LinkExists) => ShmemConf::new().flink(shmem_flink).open().unwrap(),
        Err(e) => {
            eprintln!("Unable to create or open shmem flink {shmem_flink} : {e}");
            return;
        }
    };

    let mut raw_ptr = shmem.as_ptr();
    let is_init: &mut AtomicU8;

    unsafe {
        is_init = &mut *(raw_ptr as *mut u8 as *mut AtomicU8);
        raw_ptr = raw_ptr.add(8);
    };

    // Initialize or wait for initialized mutex
    let mutex = if shmem.is_owner() {
        is_init.store(0, Ordering::Relaxed);
        // Initialize the mutex
        let (lock, _bytes_used) = unsafe {
            Mutex::new(
                raw_ptr,                                    // Base address of Mutex
                raw_ptr.add(Mutex::size_of(Some(raw_ptr))), // Address of data protected by mutex
            )
            .unwrap()
        };
        is_init.store(1, Ordering::Relaxed);
        lock
    } else {
        // wait until mutex is initialized
        while is_init.load(Ordering::Relaxed) != 1 {}
        // Load existing mutex
        let (lock, _bytes_used) = unsafe {
            Mutex::from_existing(
                raw_ptr,                                    // Base address of Mutex
                raw_ptr.add(Mutex::size_of(Some(raw_ptr))), // Address of data  protected by mutex
            )
            .unwrap()
        };
        lock
    };

    // Loop until mutex data reaches 10
    loop {
        // Scope where mutex will be locked
        {
            let mut guard = mutex.lock().unwrap();
            // Cast mutex data to &mut u8
            let val: &mut u8 = unsafe { &mut **guard };
            if *val > 5 {
                println!("[thread#{thread_num}] done !");
                return;
            }

            // Print contents and increment value
            println!("[thread#{}] Val : {}", thread_num, *val);
            *val += 1;

            // Hold lock for a second
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        // Timeout this thread for a second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
