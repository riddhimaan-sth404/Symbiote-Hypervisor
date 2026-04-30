mod kvm;
mod memory;
mod vcpu;
mod guest_loader;
mod reflex_engine;

use nix::mount::{mount, MsFlags};
use std::process::Command;
use std::path::Path;
use guest_loader::GuestBinary;

fn main() {
    println!("========================================");
    println!("     SYMBIOTE OS - RING -1 INITIALIZED  ");
    println!("========================================");

    // 1. Mount Essential Kernel Filesystems
    prepare_filesystem();

    // 2. Hardware Check: Is KVM available?
    if !Path::new("/dev/kvm").exists() {
        println!("[!] FATAL: KVM not found. Check kernel config.");
        loop {} // Halt if we can't virtualize
    }
    println!("[+] KVM Hypervisor detected.");

    // 3. Setup Networking (Host Side)
    setup_host_network();

    // 4. Initialize Hypervisor and Load Guest Code
    println!("[*] Initializing Symbiote Hypervisor...");
    
    // Create VM instance
    let hypervisor = kvm::SymbioteHypervisor::new();
    
    // Setup guest memory (64MB)
    let guest_mem_region = memory::setup_guest_memory(&hypervisor.vm, 0x0, 64 * 1024 * 1024);
    
    // Load guest code
    let guest_binary = GuestBinary::test_stub();
    guest_binary.load_into_memory(guest_mem_region.host_addr, 0x1000);
    
    // Setup vCPU with guest memory info
    let mut vcpu = vcpu::SymbioteCPU::new(&hypervisor.vm);
    vcpu.setup_registers();
    
    // 5. Execute Hypervisor Loop
    println!("[*] Launching Symbiote Sentry Engine...");
    vcpu.run_loop();
    
    println!("[!] Hypervisor loop exited. System halted.");
}

fn prepare_filesystem() {
    let none: Option<&str> = None;
    
    // Mount /proc, /sys, and /dev
    mount(Some("proc"), "/proc", Some("proc"), MsFlags::empty(), none).ok();
    mount(Some("sysfs"), "/sys", Some("sysfs"), MsFlags::empty(), none).ok();
    // devtmpfs should be automounted by kernel, but we ensure it here
    mount(Some("devtmpfs"), "/dev", Some("devtmpfs"), MsFlags::empty(), none).ok();
    
    println!("[+] Kernel filesystems mounted.");
}

fn setup_host_network() {
    // Bring up loopback
    let _ = Command::new("/bin/ifconfig").args(["lo", "127.0.0.1", "up"]).status();
    println!("[+] Networking initialized.");
}
