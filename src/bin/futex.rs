use libc::{c_int, c_void, pid_t};
use std::{sync::atomic::{AtomicU32, Ordering}, thread::sleep, time::Duration, ptr, mem::size_of};

// futex wrappers because they aren't in libc
// todo: how much of this is unsafe? Could these wrappers be safe functions?
unsafe fn futex_wake(addr: *const AtomicU32, nr: c_int) -> i64 {
    libc::syscall(
        libc::SYS_futex,
        addr,
        libc::FUTEX_WAKE,
        nr,
        libc::PT_NULL,
        libc::PT_NULL,
        0u32,
    )
}

unsafe fn futex_wait(addr: *const AtomicU32, val: c_int) -> i64 {
    libc::syscall(
        libc::SYS_futex,
        addr,
        libc::FUTEX_WAIT,
        val,
        libc::PT_NULL,
        libc::PT_NULL,
        0u32,
    )
}

// Task 1: Implement Semaphore
struct Sem(AtomicU32);

impl Sem {
    // Initialize the semaphore with a given value
    pub fn new(init: u32) -> Self {
        Sem(AtomicU32::new(init))
    }

    // Try to decrement the semaphore. If it's larger than 0, just decrement.
    // Otherwise, sleep until it's larger than 0 and then try decrementing it.
    pub fn down(&mut self) {
        // Use a loop because there could be a race
        loop {
            // todo: How strong an ordering guarantee do we actually need here?
            let val = self.0.load(Ordering::Acquire);
            if val > 0 {
                if self
                    .0
                    .compare_exchange(val, val - 1, Ordering::AcqRel, Ordering::Acquire)
                    == Ok(val)
                {
                    // Compare and exchange succeeded
                    break;
                }
            } else {
                unsafe {
                    futex_wait(&self.0, 0);
                }
            }
        }
    }

    // Increment the counter and wake one waiting thread
    pub fn up(&mut self) {
        // Increment the semaphore unconditionally
        let prev = self.0.fetch_add(1, Ordering::AcqRel);

        // If the old value was zero, somebody might be waiting. Therefore wake them up.
        if prev == 0 {
            unsafe {
                futex_wake(&self.0, 1);
            }
        }
    }
}

// Task 2: Implement Bounded Buffer

const ARRAY_SIZE: usize = 3;

struct BoundedBuffer<T> {
    // Two semaphores for the number of empty slots and the number of valid elements
    slots: Sem,
    elements: Sem,

    // Another semaphore as a binary semaphore for synchronizing access to data and metadata
    lock: Sem,

    read_idx: usize, // next slot to read
    write_idx: usize, // next slot to write

    data: [Option<T>; ARRAY_SIZE],
}

// Restricting to copyable values because it's a headache trying to do this otherwise and that's not really the point
impl <T: Copy> BoundedBuffer<T> {
    pub fn new() -> Self {
        BoundedBuffer {
            // Count number of empty slots (initially ARRAY_SIZE) and valid elements (initially 0)
            slots: Sem::new(ARRAY_SIZE.try_into().expect("could not convert usize to u32")),
            elements: Sem::new(0),
            
            // 1 means the mutex is free
            lock: Sem::new(1),
            
            // Start reading and writing at 0
            read_idx: 0,
            write_idx: 0,
            
            data: [None; ARRAY_SIZE],
        }
    }

    // blocks until there is an item to get
    pub fn get(&mut self) -> T {
        // Ensure there is an element to get
        self.elements.down();

        let ret: Option<T>;

        // Critical section, protected by lock
        {
            self.lock.down();

            ret = self.data[self.read_idx].take();
            self.read_idx = (self.read_idx + 1) % ARRAY_SIZE;

            self.lock.up();
        }

        // More slots are now empty, so increase the slots semaphore which may wake other threads
        self.slots.up();

        ret.unwrap()
    }

    pub fn put(&mut self, val: T) {
        // Ensure there is space for the element
        self.slots.down();

        // Critical section
        {
            self.lock.down();
            
            self.data[self.write_idx] = Some(val);
            self.write_idx = (self.write_idx + 1) % ARRAY_SIZE;

            self.lock.up();
        }

        self.elements.up();
    }
}

// Task 3: Use the Bounded Buffer

fn main() {
    let child: pid_t;
    let ready: &mut Sem;
    let buf: &mut BoundedBuffer<u32>;

    // Make a shared region of memory for the buffer
    unsafe {
        let shared_mem = libc::mmap(libc::PT_NULL as *mut c_void, 4096, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_ANONYMOUS | libc::MAP_SHARED, -1, 0);
        if shared_mem == libc::MAP_FAILED {
            panic!("Could not mmap");
        }

        let ready_location = shared_mem;
        ptr::write(ready_location as *mut Sem, Sem::new(0));
        ready = &mut *(ready_location as *mut Sem);

        // The correct way to do this would probably be to have the semaphore wrapped up with the buffer in a struct or something.
        // As it is right now it's really bad because rust doesn't know that the reference isn't actually initialized.
        let buf_location = shared_mem.add(size_of::<Sem>());
        buf = &mut *(buf_location as *mut BoundedBuffer<u32>);

        // Fork to test the synchronization
        child = libc::fork();
    }

    if child != 0 {
        // Parent

        println!("Parent: Waiting for child to initialize buffer...");

        ready.down();

        println!("Parent: Child has initialized buffer. Reading from buffer...");

        loop {
            let val = buf.get();
            println!("Parent: {}", val);
            if val == 5 {
                break
            }
        }
        
    } else {
        // Child

        sleep(Duration::from_secs(1));

        println!("Child: Initializing buffer...");

        unsafe {
            ptr::write(buf as *mut BoundedBuffer<u32>, BoundedBuffer::<u32>::new());
        }

        println!("Child: Initialized buffer.");

        ready.up();

        println!("Child: Writing to buffer...");

        for n in 1..6 {
            sleep(Duration::from_secs(1));
            buf.put(n);
        }
    }
}
