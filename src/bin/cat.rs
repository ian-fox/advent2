use std::env;
use std::process;

#[macro_use]
extern crate advent_2;

const BUF_SIZE: usize = 4096;
const STDOUT_FD: libc::c_int = 1;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: cat <path> ...");
        process::exit(1);
    }

    // Keep track of if we should be exiting with an error status
    let mut is_err = false;

    unsafe {
        let mut buf = [0u8; BUF_SIZE];
        let buf_void = buf.as_mut_ptr() as *mut libc::c_void;

        for arg in args.iter().skip(1) {
            let path: *const u8 = arg.as_bytes().as_ptr();

            let fd = libc::open(path as *const u8, libc::O_RDONLY);

            debug!("fd is {}", fd);

            if fd < 0 {
                eprintln!("could not open file: {}", arg);
                is_err = true;
                continue;
            }

            let size_read = libc::read(fd, buf_void, BUF_SIZE.try_into().unwrap());

            debug!("read {} bytes", size_read);

            if size_read < 0 {
                eprintln!("could not read file: {}", arg);
                is_err = true;
            } else {
                let size_write = libc::write(STDOUT_FD, buf_void, size_read.try_into().unwrap());

                debug!("wrote {} bytes", size_write);

                if size_write < 0 {
                    eprintln!("could not write to stdout");
                    is_err = true;
                }
            }

            let close = libc::close(fd);

            debug!("close returned {}", close);

            if close != 0 {
                is_err = true;
            }
        }

        if is_err {
            process::exit(1);
        }
    }
}
