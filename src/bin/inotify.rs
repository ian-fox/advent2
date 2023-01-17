use libc::{c_char, c_void, close, inotify_event, IN_ACCESS, IN_CLOSE, IN_OPEN};
use std::collections::BTreeMap;
use std::ffi::{CStr, CString};
use std::io::Error;
use std::mem::size_of;
use std::process;

#[macro_use]
extern crate advent_2;

const BUF_SIZE: usize = 4096;
static mut BUF: [u8; BUF_SIZE] = [0u8; BUF_SIZE];

fn error_exit(msg: &str) {
    println!("Error in {}: {:?}", msg, Error::last_os_error());
    process::exit(1);
}
fn main() {
    let flag_names = BTreeMap::from([
        (IN_ACCESS, "IN_ACCESS"),
        (IN_OPEN, "IN_OPEN"),
        (IN_CLOSE, "IN_CLOSE"),
    ]);

    unsafe {
        debug!("init inotify");
        let inotify_fd = libc::inotify_init();
        if inotify_fd < 0 {
            error_exit("inotify_init");
        }

        debug!("add watch");
        let path = CString::new(".").unwrap();
        if libc::inotify_add_watch(
            inotify_fd,
            path.as_ptr() as *const c_char,
            IN_OPEN | IN_ACCESS | IN_CLOSE,
        ) < 0
        {
            error_exit("inotify_add_watch");
        }

        // print events
        loop {
            let length = libc::read(inotify_fd, BUF.as_ptr() as *mut c_void, BUF_SIZE);
            if length < 0 {
                break;
            }

            debug!("got event[s]");

            let mut event: *const inotify_event =
                BUF.as_ptr() as *const c_void as *const inotify_event;
            while event < (&BUF as *const u8).add(BUF_SIZE) as *const inotify_event {
                // If there's no wd it's not actually an event, just zeroed memory
                if (*event).wd == 0 {
                    break;
                }

                if (*event).len > 0 {
                    let name: *mut u8 = event.add(1) as *mut u8;
                    let name_str = CStr::from_ptr(name);
                    print!(
                        "./{} ",
                        String::from_utf8_lossy(name_str.to_bytes()).to_string(),
                    )
                } else {
                    print!(". ");
                }

                print!(
                    "{:?}",
                    &[IN_ACCESS, IN_OPEN, IN_CLOSE]
                        .iter()
                        .filter(|flag| (**flag & (*event).mask) > 0)
                        .map(|flag| flag_names.get(flag).expect("got an unexpected flag"))
                        .collect::<Vec<&&str>>()
                );
                print!("\n");

                event = event.add(size_of::<inotify_event>() + (*event).len as usize);
            }
        }

        close(inotify_fd);
    }
}
