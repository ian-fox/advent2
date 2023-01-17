use libc::{sighandler_t};
use libc::{
    self, c_int, c_void, siginfo_t,ucontext_t,
};
use std::arch::asm;
use std::io::Error;
use std::process::{self, Command};
use std::ptr;
use std::mem;

#[macro_use]
extern crate advent_2;

fn error_exit(msg: &str) {
    println!("Error in {}: {:?}", msg, Error::last_os_error());
    process::exit(1);
}

static mut DO_EXIT: bool = false;
static mut PAGE_SIZE: i64 = -1;

fn sa_sigint(_signum: c_int, _info: *const siginfo_t, _context: *const c_void) {
    debug!("sa_sigsegv");
    
    // todo: do we need this to be volatile?
    unsafe {
        ptr::write_volatile(&mut DO_EXIT, true);
    }
}

unsafe fn sa_sigsegv(_signum: c_int, info: *const siginfo_t, _context: *const c_void) {
    println!("sa_sigsegv: si_addr = 0x{:?}", (*info).si_addr());

    // Calculate page address
    let mut addr = (*info).si_addr();
    addr = (addr as usize & (!(PAGE_SIZE as usize - 1))) as *mut c_void;

    // Mmap a page there
    let ret = libc::mmap(
        addr,
        PAGE_SIZE as usize,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
        -1,
        0,
    );
    if ret == libc::MAP_FAILED {
        error_exit("mmap");
    }

    println!("sa_sigsegv: mmap(PAGE_SIZE) -> 0x{:?}", addr);
}

unsafe fn sa_sigill(_signum: c_int, _info: *const siginfo_t, context: *mut c_void) {
    let ctx = context as *mut ucontext_t;

    let pc = (*ctx).uc_mcontext.pc;
    println!("sa_sigill: REG_RIP = main + 0x{:?}", pc - (main as *const () as u64));

    // jump 4 bytes forward, for demonstration purposes (fails if faulting instruction is not 4 bytes long)
    (*ctx).uc_mcontext.pc += 4;
}
fn main() {
    unsafe {
        debug!("getting page size");
        PAGE_SIZE = libc::sysconf(libc::_SC_PAGE_SIZE);
        if PAGE_SIZE == -1 {
            error_exit("sysconf");
        }

        debug!("setting handlers");
        let mut sa = libc::sigaction {
            sa_sigaction: 0 as sighandler_t,
            sa_mask: mem::zeroed(),
            sa_flags: libc::SA_SIGINFO | libc::SA_RESTART,
            sa_restorer: None,
        };
        if libc::sigemptyset(&mut sa.sa_mask) == -1 {
            error_exit("sigemptyset")
        }

        sa.sa_sigaction = sa_sigint as sighandler_t;
        if libc::sigaction(libc::SIGINT, &sa, ptr::null_mut()) != 0 {
            error_exit("sigaction for sigint");
        }

        sa.sa_sigaction = sa_sigsegv as sighandler_t;
        if libc::sigaction(libc::SIGSEGV, &sa, ptr::null_mut()) != 0 {
            error_exit("sigaction for sigsegv");
        }

        sa.sa_sigaction = sa_sigill as sighandler_t;
        if libc::sigaction(libc::SIGILL, &sa, ptr::null_mut()) != 0 {
            error_exit("sigaction for sigill");
        }

        // We generate an invalid pointer that points _somewhere_! This is
        // undefined behavior, and we only hope for the best here.
        let mut addr = 0xdeadbeef as *mut u32;

        // This will provoke a SIGSEGV
        debug!("dereferencing a bad pointer");
        *addr = 23;

        // In the original solution, two ud2 instructions are used to get to exactly 4 bytes long
        // Except I'm on ARM, so we'll use UDF.
        // This will provoke a SIGILL
        unsafe fn invalid_opcode() {
            asm!(".word 0xf7f0a000");
        }

        debug!("calling an invalid opcode");
        invalid_opcode();

        // Happy faulting, until someone sets the do_exit variable.
        // Perhaps the SIGINT handler?
        while !ptr::read_volatile(&DO_EXIT) {
            libc::sleep(1);

            debug!("doing both");
            addr = addr.wrapping_add(22559);
            *addr = 42;
            invalid_opcode();
        }


        // Like in the mmap exercise, we use pmap to show our own memory
        // map, before exiting.
        let pid: libc::pid_t = libc::getpid();

        debug!("calling pmap");
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
