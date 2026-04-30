mod kvm;
mod memory;
mod vcpu;

use nix::mount::{mount, MsFlags};
use std::process::Command;
use std::path::Path;

fn main() {
    println!("========================================");
    println!("     SYMBIOTE OS - RING -1 INITIALIZED  ");
    println!("========================================");

    // 1. Mount Essential Kernel Filesystems
    prepare_filesystem();

    // 2. Hardware Check: Is KVM available?
    if Path::new("/dev/kvm").exists() {
        println!("[+] KVM Hypervisor detected.");
    } else {
        println!("[!] FATAL: KVM not found. Check kernel config.");
        loop {} // Halt if we can't virtualize
    }

    // 3. Setup Networking (Host Side)
    setup_host_network();

    // 4. Launch the "Reflex Engine" (Your C-based KVM logic)
    // For now, we drop to a shell so you can debug.
    println!("[*] Entering System Shell...");
    let _ = Command::new("/bin/sh").spawn().expect("Failed to start shell").wait();
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
