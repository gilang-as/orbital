//! Build script for Orbital Kernel
//!
//! Compiles and embeds the userspace binary into the kernel.
//! This allows the kernel to load and execute it as a userspace process.

use std::path::PathBuf;

fn main() {
    // Phase 4.1: Use minimal userspace shell (1.2 KB, compiled for x86_64-orbital)
    let cli_binary_path = PathBuf::from("../userspace/minimal/target/x86_64-orbital/release/minimal-shell");
    
    // Verify the binary exists
    if cli_binary_path.exists() {
        println!("cargo:rustc-env=ORBITAL_CLI_PATH={}", cli_binary_path.display());
        println!("cargo:rerun-if-changed={}", cli_binary_path.display());
        println!("cargo:rustc-cfg=have_cli_binary");
        println!("cargo:warning=Embedding userspace shell ({} bytes)", 
                 std::fs::metadata(&cli_binary_path)
                     .map(|m| m.len())
                     .unwrap_or(0));
    } else {
        eprintln!("Warning: Minimal shell binary not found at {:?}", cli_binary_path);
        eprintln!("Phase 4.1: To build minimal shell:");
        eprintln!("  cd userspace/minimal && cargo build --release");
    }

    // Tell cargo to rerun if minimal shell source changes
    println!("cargo:rerun-if-changed=../userspace/minimal/src");
}

