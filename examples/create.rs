extern crate mem_file;
use mem_file::*;
use std::path::PathBuf;

struct SharedState {
    num_listenners: u32,
    message: [u8; 256],
}
//WARNING : Only do this if you know what you're doing.
unsafe impl MemFileCast for SharedState {}

fn main() {

    #[cfg(unix)]
    let lock_type = LockType::Rwlock;
    #[cfg(windows)]
    let lock_type = LockType::None;

    //Create a new shared MemFile
    let mut mem_file: MemFile = match MemFile::create(PathBuf::from("shared_mem.link"),  lock_type, 4096) {
        Ok(v) => v,
        Err(e) => {
            println!("Error : {}", e);
            println!("Failed to create MemFile...");
            return;
        }
    };

    println!("Created link file \"{}
    Backed by OS identifier : \"{}\"
    Size : 0x{:x}",
    mem_file.get_link_path().unwrap().to_string_lossy(),
    mem_file.get_real_path().unwrap(),
    mem_file.get_size());

    //Initialize the memory with default values
    {
        let mut shared_state: WriteLockGuard<SharedState> = match mem_file.wlock() {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire write lock !"),
        };

        shared_state.num_listenners = 0;
        let src = b"Welcome, we currently have 0 listenners !\x00";
        shared_state.message[0..src.len()].copy_from_slice(src);

        println!("Holding lock for 5 seconds !");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    println!("Waiting for a listenner to connect !");

    //Loop until our memory has changed
    loop {

        //Acquire read lock
        let shared_state: ReadLockGuard<SharedState> = match mem_file.rlock() {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire read lock !"),
        };

        //Check shared memory
        if shared_state.num_listenners > 0 {
            println!("We have a listenner !");
            break;
        }

        //Release the lock before sleeping
        drop(shared_state);
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    //Modify the shared memory just for fun
    {
        let mut shared_state: WriteLockGuard<SharedState> = match mem_file.wlock() {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire write lock !"),
        };

        let src = format!("Goodbye {} listenner(s) !\x00", shared_state.num_listenners);
        shared_state.message[0..src.len()].copy_from_slice(&src.as_bytes());
    }
}
