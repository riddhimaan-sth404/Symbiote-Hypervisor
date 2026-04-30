use kvm_ioctls::VcpuFd;
use kvm_bindings::{kvm_regs, kvm_sregs};

pub struct SymbioteCPU {
    pub fd: VcpuFd,
}

impl SymbioteCPU {
    pub fn new(vm: &kvm_ioctls::VmFd) -> Self {
        // Create vCPU 0
        let fd = vm.create_vcpu(0).expect("[-] Failed to create vCPU");
        println!("[+] vCPU 0 initialized.");
        SymbioteCPU { fd }
    }

    pub fn setup_registers(&self) {
        // Basic register setup (Standard x86_64 start state)
        let mut regs = kvm_regs::default();
        regs.rip = 0x1000; // Start execution at our mapped memory address
        regs.rflags = 2;   // Initial flags
        self.fd.set_regs(&regs).expect("[-] Failed to set registers");

        // Special registers (Segments, Control Registers)
        let mut sregs = self.fd.get_sregs().expect("[-] Failed to get sregs");
        sregs.cs.base = 0;
        sregs.cs.selector = 0;
        self.fd.set_sregs(&sregs).expect("[-] Failed to set sregs");
    }

    pub fn run_loop(&mut self) {
        println!("[*] Symbiote Sentry: Entering vCPU Run Loop...");
        loop {
            match self.fd.run().expect("[-] vCPU run failed") {
                kvm_ioctls::VcpuExit::IoIn(addr, _data) => {
                    println!("[>] Guest Port Read: 0x{:x}", addr);
                }
                kvm_ioctls::VcpuExit::IoOut(addr, data) => {
                    println!("[<] Guest Port Write: 0x{:x} (Data: {:?})", addr, data);
                }
                kvm_ioctls::VcpuExit::Hlt => {
                    println!("[!] Guest Halted. Suspending core.");
                    break;
                }
                exit_reason => {
                    println!("[?] Unhandled VM Exit: {:?}", exit_reason);
                    break;
                }
            }
        }
    }
}