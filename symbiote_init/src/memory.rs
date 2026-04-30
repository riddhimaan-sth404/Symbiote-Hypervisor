use kvm_bindings::kvm_userspace_memory_region;

/// Guest memory region information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GuestMemoryRegion {
    pub host_addr: u64,
    pub guest_addr: u64,
    pub size: usize,
}

impl GuestMemoryRegion {
    /// Write data into guest memory at a specific offset
    #[allow(dead_code)]
    pub fn write_at(&self, offset: u64, data: &[u8]) {
        unsafe {
            let dest = (self.host_addr + offset) as *mut u8;
            std::ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());
        }
    }

    /// Read data from guest memory at a specific offset
    #[allow(dead_code)]
    pub fn read_at(&self, offset: u64, len: usize) -> Vec<u8> {
        unsafe {
            let src = (self.host_addr + offset) as *const u8;
            std::slice::from_raw_parts(src, len).to_vec()
        }
    }
}

pub fn setup_guest_memory(vm: &kvm_ioctls::VmFd, guest_addr: u64, size: usize) -> GuestMemoryRegion {
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
    
    let host_addr_u64 = host_addr as u64;
    println!("[+] Guest memory region mapped at 0x{:x} ({}MB)", host_addr_u64, size / 1024 / 1024);
    
    GuestMemoryRegion {
        host_addr: host_addr_u64,
        guest_addr,
        size,
    }
}