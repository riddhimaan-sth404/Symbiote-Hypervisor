use kvm_ioctls::VcpuFd;
use kvm_bindings::kvm_regs;

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
        let mut exit_count = 0;
        
        loop {
            match self.fd.run().expect("[-] vCPU run failed") {
                kvm_ioctls::VcpuExit::IoIn(addr, _data) => {
                    exit_count += 1;
                    println!("[Exit #{}] Guest Port Read: 0x{:x}", exit_count, addr);
                }
                kvm_ioctls::VcpuExit::IoOut(addr, data) => {
                    exit_count += 1;
                    println!("[Exit #{}] Guest Port Write: 0x{:x} (Data: {:?})", exit_count, addr, data);
                    
                    // Debug port 0xE9 - common hypervisor debug output
                    if addr == 0xe9 && data.len() == 1 {
                        println!("  [DEBUG OUTPUT] 0x{:02x} ('{}')", data[0], 
                            if data[0] >= 32 && data[0] < 127 { 
                                data[0] as char 
                            } else { 
                                '.' 
                            }
                        );
                    }
                }
                kvm_ioctls::VcpuExit::Hlt => {
                    exit_count += 1;
                    println!("[Exit #{}] Guest Halted. Suspending core.", exit_count);
                    break;
                }
                kvm_ioctls::VcpuExit::MmioWrite(addr, data) => {
                    exit_count += 1;
                    println!("[Exit #{}] MMIO Write: 0x{:x} (Data: {:?})", exit_count, addr, data);
                }
                kvm_ioctls::VcpuExit::MmioRead(addr, _data) => {
                    exit_count += 1;
                    println!("[Exit #{}] MMIO Read: 0x{:x}", exit_count, addr);
                }
                exit_reason => {
                    exit_count += 1;
                    println!("[Exit #{}] Unhandled VM Exit: {:?}", exit_count, exit_reason);
                    // Don't break for other exits - could be legitimate
                }
            }
        }
        
        println!("[+] vCPU loop completed after {} exits", exit_count);
    }
}