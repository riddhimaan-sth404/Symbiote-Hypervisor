/// Guest Code Loader
/// Handles injection of guest binaries into guest RAM before vCPU execution
use std::ptr;

/// Simple guest binary structure
#[allow(dead_code)]
pub struct GuestBinary {
    pub code: Vec<u8>,
    pub entry_point: u64,
}

impl GuestBinary {
    /// Create a guest binary from raw bytes
    #[allow(dead_code)]
    pub fn from_bytes(code: Vec<u8>, entry_point: u64) -> Self {
        GuestBinary {
            code,
            entry_point,
        }
    }

    /// Create a minimal x86-64 test stub
    /// This is a simple infinite loop with port writes for debugging
    pub fn test_stub() -> Self {
        // x86-64 assembly:
        // 0x0: mov al, 0x42        ; Load 0x42 into AL
        // 0x2: out 0xE9, al        ; Write to debug port
        // 0x4: jmp 0x0             ; Infinite loop
        let code = vec![
            0xb0, 0x42,        // mov al, 0x42
            0xe6, 0xe9,        // out 0xe9, al
            0xeb, 0xfc,        // jmp -4 (relative jump to 0x0)
        ];

        GuestBinary {
            code,
            entry_point: 0x1000,
        }
    }

    /// Load the guest binary into guest memory
    /// 
    /// # Arguments
    /// * `guest_memory_addr` - Host address of mapped guest RAM
    /// * `offset` - Offset within guest memory to load at (typically 0x1000)
    pub fn load_into_memory(&self, guest_memory_addr: u64, offset: u64) {
        unsafe {
            let dest = (guest_memory_addr + offset) as *mut u8;
            ptr::copy_nonoverlapping(self.code.as_ptr(), dest, self.code.len());
            println!(
                "[+] Guest code loaded at 0x{:x} ({} bytes)",
                offset,
                self.code.len()
            );
        }
    }

    /// Get the size of the guest binary
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.code.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_creation() {
        let stub = GuestBinary::test_stub();
        assert!(stub.size() > 0);
        assert_eq!(stub.entry_point, 0x1000);
    }
}
