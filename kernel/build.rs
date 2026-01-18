//! Build script for Orbital Kernel
//!
//! Compiles and embeds the userspace CLI binary into the kernel.
//! This allows the kernel to load and execute the CLI as a userspace process.

use std::path::PathBuf;

fn main() {
    // Step 1: Build the userspace CLI
    // For now, we reference the pre-built binary from userspace/cli/target/
    // In a production system, this would build for the x86_64-orbital target

    let cli_binary_path = PathBuf::from("../userspace/cli/target/x86_64-apple-darwin/release/orbital-cli");
    
    // Verify the binary exists
    if cli_binary_path.exists() {
        println!("cargo:rustc-env=ORBITAL_CLI_PATH={}", cli_binary_path.display());
        println!("cargo:rerun-if-changed={}", cli_binary_path.display());
        println!("cargo:rustc-cfg=have_cli_binary");
    } else {
        eprintln!("Warning: Userspace CLI binary not found at {:?}", cli_binary_path);
        eprintln!("Phase 4: Binary loader prepared but CLI binary not embedded yet");
        eprintln!("To use Phase 4, build: cd userspace/cli && cargo build --release");
    }

    // Tell cargo to rerun this script if the CLI source changes
    println!("cargo:rerun-if-changed=../userspace/cli/src");
}
