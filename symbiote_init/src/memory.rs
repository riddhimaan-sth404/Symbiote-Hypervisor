use std::num::NonZeroUsize;
use nix::sys::mman::{mmap, MapFlags, ProtFlags};
use std::os::unix::io::BorrowedFd;
use kvm_bindings::kvm_userspace_memory_region;

pub fn setup_guest_memory(vm: &kvm_ioctls::VmFd, guest_addr: u64, size: usize) {
    // 1. Map host memory using raw libc for anonymous mapping
    let host_addr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        )
    };

    if host_addr == libc::MAP_FAILED {
        panic!("Failed to mmap guest memory");
    }

    // 2. Initialize the region
    let region = kvm_userspace_memory_region {
        slot: 0,
        flags: 0,
        guest_phys_addr: guest_addr,
        memory_size: size as u64,
        userspace_addr: host_addr as u64,
    };

    unsafe {
        vm.set_user_memory_region(region).expect("Failed to set guest memory");
    }
    
    println!("[+] Guest memory region mapped at 0x{:x} ({}MB)", host_addr as u64, size / 1024 / 1024);
}