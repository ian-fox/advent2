use libc::c_void;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::process::Command;

const PERSISTENCE_PATH: &str = "mmap.persistent";

// If you actually wanted multiple persistent values you'd want to put them in this struct
// rather than instantiating a bunch of copies of the struct since that'd mean a bunch of new pages
#[repr(align(4096))]
struct Persistent(i32);

static mut FOOBAR: Persistent = Persistent(26);
static mut BARFOO: i32 = 42;

fn main() {
    // Setup persistence of FOOBAR
    let path = Path::new(PERSISTENCE_PATH);

    unsafe {
        if !path.exists() {
            let mut f = File::create(path).expect("could not create persistence file");

            let foobar_bytes = ::std::slice::from_raw_parts(
                (&FOOBAR as *const Persistent) as *const u8,
                ::std::mem::size_of::<Persistent>(),
            );
            f.write_all(foobar_bytes)
                .expect("could not write to persistence file");
        }

        // Now replace the contents of FOOBAR with a file!
        // It's just easier to get the file from libc than try to figure out how to cast a BorrowedFd to i32.
        let fd = libc::open(
            PERSISTENCE_PATH.bytes().collect::<Vec<u8>>().as_ptr() as *const u8,
            libc::O_RDWR,
        );
        if fd < 0 {
            panic!(
                "could not open persistence file: {}",
                Error::last_os_error()
            );
        }

        let map = libc::mmap(
            &mut FOOBAR as *mut Persistent as *mut c_void,
            ::std::mem::size_of::<Persistent>(),
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED | libc::MAP_FIXED,
            fd,
            0,
        );
        if map == libc::MAP_FAILED {
            panic!("map failed: {}", Error::last_os_error());
        }

        libc::close(fd);

        FOOBAR.0 += 1;
        BARFOO += 1;

        println!("foobar ({:?}): {}", &mut FOOBAR.0 as *mut i32, FOOBAR.0);
        println!("barfoo ({:?}): {}", &mut BARFOO as *mut i32, BARFOO);

        // Print the mappings
        let pid: libc::pid_t = libc::getpid();

        println!(
            "{}",
            String::from_utf8(
                Command::new("pmap")
                    .arg(format!("{}", pid))
                    .output()
                    .expect("could not call pmap")
                    .stdout
            )
            .expect("could not convert pmap output from bytes")
        );
    }
}
