/// Reflex Engine - Memory Introspection & Forensics Module
/// Provides real-time memory analysis, threat hunting, and forensic capabilities
/// Operating transparently at Ring -1 without guest OS awareness

use crate::memory::GuestMemoryRegion;

/// VM Exit analysis records
#[derive(Debug, Clone)]
pub struct VMExitRecord {
    pub exit_number: u32,
    pub exit_type: String,
    pub rip: u64,
    pub details: String,
    pub timestamp: u64,
}

/// Guest memory signature for threat detection
#[derive(Debug, Clone)]
pub struct MemorySignature {
    pub pattern: Vec<u8>,
    pub offset: u64,
    pub severity: ThreatSeverity,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// The Reflex Engine - Core forensics and analysis component
pub struct ReflexEngine {
    guest_memory: GuestMemoryRegion,
    exit_history: Vec<VMExitRecord>,
    threat_signatures: Vec<MemorySignature>,
    max_history_size: usize,
}

impl ReflexEngine {
    /// Create a new Reflex Engine instance
    pub fn new(guest_memory: GuestMemoryRegion, max_history: usize) -> Self {
        ReflexEngine {
            guest_memory,
            exit_history: Vec::new(),
            threat_signatures: Vec::new(),
            max_history_size: max_history,
        }
    }

    /// Record a VM Exit event
    pub fn record_exit(&mut self, exit_number: u32, exit_type: &str, details: &str) {
        let record = VMExitRecord {
            exit_number,
            exit_type: exit_type.to_string(),
            rip: 0, // Would be populated from vCPU state in production
            details: details.to_string(),
            timestamp: 0, // Would use actual timestamp
        };

        self.exit_history.push(record);
        
        // Maintain max history size
        if self.exit_history.len() > self.max_history_size {
            self.exit_history.remove(0);
        }
    }

    /// Scan guest memory for known threat signatures
    pub fn scan_for_threats(&self, start_offset: u64, size: usize) -> Vec<MemorySignature> {
        let mut detected_threats = Vec::new();

        for signature in &self.threat_signatures {
            // Search for the pattern in the specified memory range
            let memory_data = self.guest_memory.read_at(start_offset, size);
            
            if let Some(pos) = self.find_pattern(&memory_data, &signature.pattern) {
                let mut threat = signature.clone();
                threat.offset = start_offset + pos as u64;
                detected_threats.push(threat);
            }
        }

        detected_threats
    }

    /// Dump a specific memory range for forensic analysis
    pub fn dump_memory(&self, offset: u64, size: usize) -> Vec<u8> {
        self.guest_memory.read_at(offset, size)
    }

    /// Analyze guest memory for suspicious patterns
    pub fn analyze_memory_region(&self, offset: u64, size: usize) -> MemoryAnalysis {
        let data = self.dump_memory(offset, size);
        
        MemoryAnalysis {
            offset,
            size,
            entropy: calculate_entropy(&data),
            null_byte_ratio: count_null_bytes(&data) as f32 / data.len() as f32,
            executable_hints: detect_executable_hints(&data),
        }
    }

    /// Register a threat signature for detection
    pub fn register_signature(&mut self, signature: MemorySignature) {
        self.threat_signatures.push(signature);
    }

    /// Get exit history statistics
    pub fn get_exit_statistics(&self) -> ExitStatistics {
        let mut exit_counts = std::collections::HashMap::new();
        
        for record in &self.exit_history {
            *exit_counts.entry(record.exit_type.clone()).or_insert(0) += 1;
        }

        ExitStatistics {
            total_exits: self.exit_history.len(),
            unique_exit_types: exit_counts.len(),
            exit_type_counts: exit_counts,
        }
    }

    /// Helper: Find pattern in data
    fn find_pattern(&self, haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|w| w == needle)
    }
}

/// Memory analysis results
#[derive(Debug)]
pub struct MemoryAnalysis {
    pub offset: u64,
    pub size: usize,
    pub entropy: f32,
    pub null_byte_ratio: f32,
    pub executable_hints: bool,
}

/// Exit statistics
#[derive(Debug)]
pub struct ExitStatistics {
    pub total_exits: usize,
    pub unique_exit_types: usize,
    pub exit_type_counts: std::collections::HashMap<String, usize>,
}

/// Calculate Shannon entropy of data (higher = more random)
fn calculate_entropy(data: &[u8]) -> f32 {
    let mut frequencies = [0u32; 256];
    
    for &byte in data {
        frequencies[byte as usize] += 1;
    }

    let len = data.len() as f32;
    let mut entropy = 0.0f32;

    for freq in frequencies.iter() {
        if *freq > 0 {
            let p = *freq as f32 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Count null bytes in data
fn count_null_bytes(data: &[u8]) -> usize {
    data.iter().filter(|&&b| b == 0).count()
}

/// Detect hints of executable code (common x86 instructions, jumps, etc.)
fn detect_executable_hints(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }

    // Look for common x86-64 instruction patterns
    let suspicious_patterns = [
        &[0x48, 0x89, 0xc0][..],  // mov rax, rax
        &[0x90][..],              // nop
        &[0xeb][..],              // jmp (short)
        &[0xe9][..],              // jmp (far)
        &[0xff, 0x25][..],        // jmp [rip+imm32]
        &[0xcc][..],              // int3 (breakpoint)
    ];

    for pattern in &suspicious_patterns {
        if let Some(_) = data.windows(pattern.len()).position(|w| w == *pattern) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_calculation() {
        let low_entropy = vec![0x00, 0x00, 0x00, 0x00];
        let high_entropy = vec![0x00, 0xFF, 0x55, 0xAA];
        
        let low_e = calculate_entropy(&low_entropy);
        let high_e = calculate_entropy(&high_entropy);
        
        assert!(high_e > low_e);
    }

    #[test]
    fn test_null_byte_counting() {
        let data = vec![0x00, 0x01, 0x00, 0x02];
        assert_eq!(count_null_bytes(&data), 2);
    }
}
