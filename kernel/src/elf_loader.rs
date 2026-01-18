//! ELF Binary Format Loader
//!
//! Phase 5: Parses ELF headers and extracts binary entry point and sections.
//! Supports x86_64 ELF LSB format (little-endian, 64-bit).
//!
//! ELF Header Format:
//! ```
//! Offset  Field           Size    Purpose
//! 0x00    Magic           4       0x7f, 'E', 'L', 'F'
//! 0x04    Class           1       1=32-bit, 2=64-bit
//! 0x05    Data            1       1=LSB (little-endian), 2=MSB (big-endian)
//! 0x06    Version         1       Should be 1
//! 0x07    OS/ABI          1       0=System V, 3=Linux
//! 0x08    ABI Version     1       0
//! 0x09    Padding         7       Reserved
//! 0x10    Type            2       2=executable
//! 0x12    Machine         2       0x3e=x86_64
//! 0x14    Version         4       Should be 1
//! 0x18    Entry Point     8       Virtual address of entry point (_start)
//! 0x20    Prog Header Off 8       Offset to first program header
//! 0x28    Section Hdr Off 8       Offset to first section header
//! 0x30    Flags           4       Machine-specific flags
//! 0x34    ELF Header Size 2       Should be 52 or 64
//! 0x36    Prog Hdr Size   2       Size of each program header entry
//! 0x38    Prog Hdr Count  2       Number of program headers
//! 0x3A    Sect Hdr Size   2       Size of each section header entry
//! 0x3C    Sect Hdr Count  2       Number of section headers
//! 0x3E    String Index    2       Section header string table index
//! ```

/// ELF magic number
const ELF_MAGIC: &[u8; 4] = b"\x7fELF";

/// ELF file class: 64-bit
const ELF_CLASS_64BIT: u8 = 2;

/// ELF data encoding: little-endian
const ELF_DATA_LSB: u8 = 1;

/// ELF file type: executable
const ELF_TYPE_EXECUTABLE: u16 = 2;

/// ELF machine type: x86_64
const ELF_MACHINE_X86_64: u16 = 0x3e;

/// ELF format error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfError {
    /// Invalid magic number (not an ELF file)
    BadMagic,
    /// Wrong ELF class (not 64-bit)
    BadClass,
    /// Wrong encoding (not little-endian)
    BadEncoding,
    /// Wrong file type (not executable)
    BadType,
    /// Wrong machine type (not x86_64)
    BadMachine,
    /// Binary too small to contain valid ELF header
    TooSmall,
    /// Version mismatch
    BadVersion,
}

/// Parsed ELF executable information (minimal)
#[derive(Debug, Clone, Copy)]
pub struct ElfInfo {
    /// Virtual address where execution should start
    pub entry_point: u64,
    /// Size of the entire binary
    pub size: u64,
}

/// Parse ELF header from a binary blob
///
/// Validates ELF magic, format flags, and extracts entry point.
/// Only validates header, does not load sections into memory.
///
/// # Arguments
/// * `binary` - Raw binary data containing ELF executable
///
/// # Returns
/// * `Ok(ElfInfo)` - Successfully parsed ELF information
/// * `Err(ElfError)` - Invalid ELF file or format mismatch
pub fn parse_elf(binary: &[u8]) -> Result<ElfInfo, ElfError> {
    // Minimum size for ELF header
    if binary.len() < 64 {
        return Err(ElfError::TooSmall);
    }

    // Check magic number
    if &binary[0..4] != ELF_MAGIC {
        return Err(ElfError::BadMagic);
    }

    // Check class (64-bit)
    if binary[4] != ELF_CLASS_64BIT {
        return Err(ElfError::BadClass);
    }

    // Check encoding (little-endian)
    if binary[5] != ELF_DATA_LSB {
        return Err(ElfError::BadEncoding);
    }

    // Check version
    if binary[6] != 1 {
        return Err(ElfError::BadVersion);
    }

    // Check file type (must be executable)
    let file_type = u16::from_le_bytes([binary[16], binary[17]]);
    if file_type != ELF_TYPE_EXECUTABLE {
        return Err(ElfError::BadType);
    }

    // Check machine type (must be x86_64)
    let machine_type = u16::from_le_bytes([binary[18], binary[19]]);
    if machine_type != ELF_MACHINE_X86_64 {
        return Err(ElfError::BadMachine);
    }

    // Extract entry point (at offset 0x18, 8 bytes, little-endian)
    let entry_point_bytes = &binary[0x18..0x20];
    let entry_point = u64::from_le_bytes([
        entry_point_bytes[0],
        entry_point_bytes[1],
        entry_point_bytes[2],
        entry_point_bytes[3],
        entry_point_bytes[4],
        entry_point_bytes[5],
        entry_point_bytes[6],
        entry_point_bytes[7],
    ]);

    Ok(ElfInfo {
        entry_point,
        size: binary.len() as u64,
    })
}

/// Validate an ELF header without full parsing
///
/// Quick check to ensure binary is a valid ELF executable.
/// Useful for early validation before memory allocation.
pub fn is_valid_elf(binary: &[u8]) -> bool {
    parse_elf(binary).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_magic() {
        let bad_magic = b"NotELF\x00\x00";
        assert_eq!(parse_elf(bad_magic), Err(ElfError::BadMagic));
    }

    #[test]
    fn test_too_small() {
        let small = b"ELF";
        assert_eq!(parse_elf(small), Err(ElfError::TooSmall));
    }

    #[test]
    fn test_valid_minimal_elf() {
        // Construct minimal valid ELF header for testing
        let mut header = [0u8; 64];

        // Magic
        header[0..4].copy_from_slice(ELF_MAGIC);
        // Class: 64-bit
        header[4] = ELF_CLASS_64BIT;
        // Encoding: little-endian
        header[5] = ELF_DATA_LSB;
        // Version
        header[6] = 1;
        // Type: executable
        header[16..18].copy_from_slice(&ELF_TYPE_EXECUTABLE.to_le_bytes());
        // Machine: x86_64
        header[18..20].copy_from_slice(&ELF_MACHINE_X86_64.to_le_bytes());
        // Entry point
        let entry = 0x1000u64;
        header[0x18..0x20].copy_from_slice(&entry.to_le_bytes());

        let result = parse_elf(&header);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().entry_point, 0x1000);
    }
}
