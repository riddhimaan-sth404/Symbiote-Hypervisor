/// Binary Loader - Loads guest payloads from initramfs into guest RAM
/// Supports multiple payload formats and flexible memory injection

use std::fs;
use std::path::Path;

/// Guest payload descriptor
#[derive(Debug, Clone)]
pub struct GuestPayload {
    pub name: String,
    pub data: Vec<u8>,
    pub load_address: u64,
    pub entry_point: u64,
    pub payload_type: PayloadType,
}

/// Types of guest payloads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    DebugStub,       // Simple debugging code (writes to port)
    Bootloader,      // Boot code (minimal kernel entry)
    TestProgram,     // Complete test program
    Custom,          // User-defined binary
}

impl GuestPayload {
    /// Load a payload from a file path
    pub fn from_file(path: &str, payload_type: PayloadType) -> Result<Self, String> {
        let data = fs::read(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;

        let name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(GuestPayload {
            name,
            data,
            load_address: 0x1000,      // Standard guest memory start
            entry_point: 0x1000,
            payload_type,
        })
    }

    /// Create a built-in debug stub
    pub fn debug_stub() -> Self {
        // x86-64 debug stub: loop writing 0x42 to port 0xE9
        // mov al, 0x42
        // out 0xe9, al
        // jmp loop (jump back -4 bytes)
        let data = vec![
            0xb0, 0x42,        // mov al, 0x42
            0xe6, 0xe9,        // out 0xe9, al
            0xeb, 0xfc,        // jmp -4
        ];

        GuestPayload {
            name: "debug_stub".to_string(),
            data,
            load_address: 0x1000,
            entry_point: 0x1000,
            payload_type: PayloadType::DebugStub,
        }
    }

    /// Create a bootloader stub
    pub fn bootloader() -> Self {
        // Minimal bootloader:
        // 1. Write boot marker to port
        // 2. Set up minimal state
        // 3. Halt
        let data = vec![
            0xb0, 0x01,        // mov al, 0x01
            0xe6, 0xe9,        // out 0xe9, al   (0x01 = boot marker)
            0xf4,              // hlt
        ];

        GuestPayload {
            name: "bootloader".to_string(),
            data,
            load_address: 0x1000,
            entry_point: 0x1000,
            payload_type: PayloadType::Bootloader,
        }
    }

    /// Create a test program that exercises multiple instructions
    pub fn test_program() -> Self {
        // Test program covering various instruction types
        let data = vec![
            // Initialize
            0x48, 0xc7, 0xc0, 0x00, 0x00, 0x00, 0x00,  // mov rax, 0
            0x48, 0xc7, 0xc1, 0x0a, 0x00, 0x00, 0x00,  // mov rcx, 10 (loop count)
            
            // loop_start:
            0x48, 0xff, 0xc0,                            // inc rax
            0xb0, 0x41,                                  // mov al, 0x41 ('A')
            0xe6, 0xe9,                                  // out 0xe9, al
            0x48, 0xff, 0xc9,                            // dec rcx
            0x75, 0xf3,                                  // jnz loop_start (back 13 bytes)
            
            // Done
            0xf4,                                        // hlt
        ];

        GuestPayload {
            name: "test_program".to_string(),
            data,
            load_address: 0x1000,
            entry_point: 0x1000,
            payload_type: PayloadType::TestProgram,
        }
    }

    /// Create custom payload from raw bytes
    pub fn custom(name: &str, data: Vec<u8>, entry_point: u64) -> Self {
        GuestPayload {
            name: name.to_string(),
            data,
            load_address: 0x1000,
            entry_point,
            payload_type: PayloadType::Custom,
        }
    }

    /// Load payload into guest memory (host-accessible address)
    pub fn load_into_memory(&self, guest_memory_host_addr: u64, offset: u64) -> Result<(), String> {
        let dest_addr = guest_memory_host_addr + offset;

        unsafe {
            let dest = dest_addr as *mut u8;
            
            // Validate memory access
            if dest.is_null() {
                return Err("Invalid memory address".to_string());
            }

            // Copy payload to guest memory
            std::ptr::copy_nonoverlapping(self.data.as_ptr(), dest, self.data.len());
        }

        println!(
            "[+] Loaded {} ({} bytes) at guest:0x{:x} (host:0x{:x})",
            self.name,
            self.data.len(),
            offset,
            dest_addr
        );

        Ok(())
    }

    /// Get payload size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Get payload info
    pub fn info(&self) -> String {
        format!(
            "{}: {} bytes, entry:0x{:x}, type:{:?}",
            self.name, self.data.len(), self.entry_point, self.payload_type
        )
    }

    /// Verify payload is within memory bounds
    pub fn validate_memory_fit(&self, max_size: usize) -> Result<(), String> {
        if self.load_address + self.data.len() as u64 > max_size as u64 {
            return Err(format!(
                "Payload too large: 0x{:x} + {} > {}",
                self.load_address, self.data.len(), max_size
            ));
        }
        Ok(())
    }
}

/// Batch loader for multiple payloads
pub struct PayloadBatcher {
    payloads: Vec<GuestPayload>,
}

impl PayloadBatcher {
    pub fn new() -> Self {
        PayloadBatcher {
            payloads: Vec::new(),
        }
    }

    /// Add payload to batch
    pub fn add(&mut self, payload: GuestPayload) {
        self.payloads.push(payload);
    }

    /// Load all payloads into guest memory
    pub fn load_all(&self, guest_memory_host_addr: u64) -> Result<(), String> {
        let mut offset = 0u64;

        for payload in &self.payloads {
            payload.load_into_memory(guest_memory_host_addr, offset)?;
            offset += payload.data.len() as u64;
            offset = (offset + 0xFF) & !0xFF; // Align to 256 bytes
        }

        println!("[+] All {} payloads loaded successfully", self.payloads.len());
        Ok(())
    }

    /// Get first payload's entry point
    pub fn get_entry_point(&self) -> Option<u64> {
        self.payloads.first().map(|p| p.entry_point)
    }

    /// List all loaded payloads
    pub fn list(&self) {
        println!("[*] Loaded {} payloads:", self.payloads.len());
        for payload in &self.payloads {
            println!("    - {}", payload.info());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_stub_creation() {
        let stub = GuestPayload::debug_stub();
        assert_eq!(stub.size(), 6);
        assert_eq!(stub.payload_type, PayloadType::DebugStub);
    }

    #[test]
    fn test_bootloader_creation() {
        let bootloader = GuestPayload::bootloader();
        assert_eq!(bootloader.size(), 5);
        assert_eq!(bootloader.payload_type, PayloadType::Bootloader);
    }

    #[test]
    fn test_test_program_creation() {
        let program = GuestPayload::test_program();
        assert!(program.size() > 20);
        assert_eq!(program.payload_type, PayloadType::TestProgram);
    }

    #[test]
    fn test_custom_payload() {
        let custom = GuestPayload::custom("test", vec![0x90, 0x90], 0x1000);
        assert_eq!(custom.size(), 2);
        assert_eq!(custom.payload_type, PayloadType::Custom);
    }
}
