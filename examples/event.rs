use raw_sync::{events::*, Timeout};
use shared_memory::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // Attempt to create a mapping or open if it already exists
    println!("Getting the shared memory mapping");
    let shmem = match ShmemConf::new().size(4096).flink("event_mapping").create() {
        Ok(m) => m,
        Err(Error::LinkExists) => ShmemConf::new().flink("event_mapping").open()?,
        Err(e) => return Err(Box::new(e)),
    };

    if shmem.is_owner() {
        //Create an event in the shared memory
        println!("Creating event in shared memory");
        let (evt, used_bytes) = unsafe { Event::new(shmem.as_ptr(), true)? };
        println!("\tUsed {used_bytes} bytes");

        println!("Launch another instance of this example to signal the event !");
        evt.wait(Timeout::Infinite)?;
        println!("\tGot signal !");
    } else {
        // Open existing event
        println!("Openning event from shared memory");
        let (evt, used_bytes) = unsafe { Event::from_existing(shmem.as_ptr())? };
        println!("\tEvent uses {used_bytes} bytes");

        println!("Signaling event !");
        evt.set(EventState::Signaled)?;
        println!("\tSignaled !");
    }

    println!("Done !");
    Ok(())
}
