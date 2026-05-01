mod kvm;
mod memory;
mod vcpu;
mod guest_loader;
mod reflex_engine;
mod binary_loader;
mod advanced_exit_handler;

use nix::mount::{mount, MsFlags};
use std::process::Command;
use std::path::Path;
use binary_loader::{GuestPayload, PayloadBatcher};

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
    
    // Setup guest memory (1GB for advanced testing)
    let guest_mem_size = 1024 * 1024 * 1024; // 1GB
    let guest_mem_region = memory::setup_guest_memory(&hypervisor.vm, 0x0, guest_mem_size);
    
    // ===== BINARY LOADER: Select Payload =====
    // Multiple payload options for testing:
    // 1. Debug stub - Simple infinite loop
    // 2. Bootloader - Boot sequence code
    // 3. Test program - Complex instruction set
    
    let mut payload_batcher = PayloadBatcher::new();
    
    // Load test program (exercises various instruction types)
    let test_payload = GuestPayload::test_program();
    payload_batcher.add(test_payload);
    
    // Add debug stub for fallback
    let debug_payload = GuestPayload::debug_stub();
    payload_batcher.add(debug_payload);
    
    // Load all payloads into guest memory
    println!("[*] Loading guest payloads...");
    match payload_batcher.load_all(guest_mem_region.host_addr) {
        Ok(()) => {
            payload_batcher.list();
            println!("[+] All payloads loaded successfully at guest:0x1000");
        }
        Err(e) => {
            println!("[!] Failed to load payloads: {}", e);
            loop {} // Halt on fatal error
        }
    }
    
    // Setup vCPU with guest memory info
    println!("[*] Configuring vCPU 0...");
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
    match mount(Some("proc"), "/proc", Some("proc"), MsFlags::empty(), none) {
        Ok(()) => println!("[+] /proc mounted"),
        Err(e) => println!("[!] Failed to mount /proc: {}", e),
    }
    
    match mount(Some("sysfs"), "/sys", Some("sysfs"), MsFlags::empty(), none) {
        Ok(()) => println!("[+] /sys mounted"),
        Err(e) => println!("[!] Failed to mount /sys: {}", e),
    }
    
    // devtmpfs should be automounted by kernel, but we ensure it here
    match mount(Some("devtmpfs"), "/dev", Some("devtmpfs"), MsFlags::empty(), none) {
        Ok(()) => println!("[+] /dev mounted"),
        Err(e) => println!("[!] Failed to mount /dev: {}", e),
    }
    
    println!("[+] Kernel filesystems initialization complete.");
}

fn setup_host_network() {
    // Bring up loopback (try multiple tools as fallback)
    let result = Command::new("ip").args(["link", "set", "lo", "up"]).status();
    if result.is_err() {
        // Fallback: ifconfig might not exist, just skip
        println!("[*] Loopback configuration skipped (ip command not available)");
    }
    println!("[+] Networking initialized.");
}
