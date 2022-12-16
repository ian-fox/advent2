// Reference: https://collaborating.tuhh.de/e-exk4/advent/-/blob/solution_2/02-clone/clone.c

use libc::{c_int, c_void};
use std::env;
use std::fs::File;
use std::io::{Error, Write};
use std::process;
use std::slice;

const BUF_SIZE: usize = 4096;
static mut BUF: [u8; BUF_SIZE] = [0u8; BUF_SIZE];
const ARG_SIZE: usize = 100;
static mut SHARED: i32 = 0;

extern "C" fn cb(arg: *mut c_void) -> c_int {
    unsafe {
        println!(
            "Hello from child! ppid: {}, pid: {}, tid: {}, uid: {}, arg: {:?}",
            libc::getppid(),
            libc::getpid(),
            libc::gettid(),
            libc::getuid(),
            arg
        );

        if arg != 0x0 as *mut c_void {
            // Install the UID map
            let mut f =
                File::create("/proc/self/uid_map").expect("Could not open /proc/self/uid_map");
            let arg_slice: &[u8] = slice::from_raw_parts(arg as *mut u8, ARG_SIZE);
            f.write_all(arg_slice).expect("Could not write UID map");

            println!("Setuid: {}", libc::setuid(0));
            println!("Now child sees uid {}", libc::getuid());
        }

        libc::sleep(1);
        println!("Child sees shared is {}", SHARED);
    }

    return 0;
}

unsafe fn my_fork(flags: c_int, arg: *mut c_void) -> c_int {
    let top_of_stack: *mut u8 = &mut BUF[BUF_SIZE - 1];
    let top_of_stack_void = top_of_stack as *mut c_void;
    let arg_void = arg as *mut c_void;

    libc::clone(cb, top_of_stack_void, flags, arg_void)
}

fn usage() {
    eprintln!("usage: clone <fork | chimera | thread | user>");
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        usage();
    }

    unsafe {
        let mut flags: c_int = 0;
        let mut arg: *mut c_void = 0 as *mut c_void;

        // Need to have the reference out here so that it doesn't get dropped before the call to clone
        let mut uid_map: Box<[u8; ARG_SIZE]>;

        match args.get(1).unwrap().as_str() {
            "fork" => {
                flags = libc::SIGCHLD;
            }
            "chimera" => {
                flags = libc::SIGCHLD | libc::CLONE_VM;
            }
            "thread" => {
                flags = libc::CLONE_VM | libc::CLONE_THREAD | libc::CLONE_SIGHAND;
            }
            "user" => {
                flags = libc::SIGCHLD | libc::CLONE_NEWUSER;

                // Make sure we're running from somewhere where it'll do something
                let uid = libc::getuid();
                if uid == 0 {
                    println!("You are already root! Try `su advent -c \"./target/debug/clone user\"`")
                }

                // Set up the uid map
                uid_map = Box::new([0u8; ARG_SIZE]);
                let uid_contents = format!("0 {} 1\n", uid);
                println!("UID map contents: {}", uid_contents);
                for (i, val) in uid_contents.as_bytes().iter().enumerate() {
                    uid_map[i] = *val;
                }
                arg = uid_map.as_mut_ptr() as *mut c_void;
            }
            _ => {
                usage();
            }
        };

        println!(
            "Hello from parent! ppid: {}, pid: {}, tid: {}, uid: {}",
            libc::getppid(),
            libc::getpid(),
            libc::gettid(),
            libc::getuid()
        );
        let child = my_fork(flags, arg);
        println!("child tid is {}", child);

        if child == -1 {
            println!("Error: {:?}", Error::last_os_error());
            process::exit(1);
        }

        println!("Parent: setting shared to 1");
        SHARED = 1;
        println!("Parent sees shared is {}", SHARED);

        // wait for the child to terminate
        // todo/bonus: wait for SIGCHLD if we set that flag when cloning, or some sort of lock if sharing memory
        libc::sleep(2);
    }
}
