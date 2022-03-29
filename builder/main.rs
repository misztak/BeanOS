/*!
A glorified build script for the OS and bootloader.

*/

use std::{env, path::PathBuf, process::Command};
use llvm_tools_build;

fn main() {
    let is_release_build = !cfg!(debug_assertions);
    let build_type = if is_release_build { "release" } else { "debug" };

    // get llvm-objcopy through llvm_tools (requires llvm-tools-preview)
    let llvm_tools = match llvm_tools_build::LlvmTools::new() {
        Ok(tools) => tools,
        Err(llvm_tools_build::Error::NotFound) => {
            panic!("llvm-tools not found. Install it through: 'rustup component add llvm-tools-preview'");
        }
        Err(err) => {
            panic!("Failed to load llvm-tools: {:?}", err);
        }
    };

    let objcopy = llvm_tools
        .tool(&llvm_tools_build::exe("llvm-objcopy"))
        .expect("llvm-objcopy not found");

    //
    // Step 1: Build the kernel
    //

    println!("1. Building BeanOS in {} mode", build_type);

    let project_root_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Environment variable 'CARGO_MANIFEST_DIR' undefined");
    let kernel_dir = PathBuf::from(&project_root_dir).join("os");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&kernel_dir);
    cmd.arg("build");
    if is_release_build { cmd.arg("--release"); }
    let cmd_status = cmd
        .status()
        .expect("Failed to run cargo to build kernel");
    assert!(cmd_status.success(), "Failed to build kernel");

    //
    // Step 2: Build the bootloader
    //

    println!("2. Building bootloader in {} mode", build_type);

    let bootloader_dir = PathBuf::from(&project_root_dir).join("bootloader");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&bootloader_dir);
    cmd.arg("build");
    if is_release_build { cmd.arg("--release"); }
    let cmd_status = cmd
        .status()
        .expect("Failed to run cargo to build bootloader");
    assert!(cmd_status.success(), "Failed to build bootloader");

    //
    // Step 3: Convert ELF file into flat binary
    //

    println!("3. Creating flat binary");

    let output_dir = bootloader_dir.join(format!("target/x86_64-bean_os_bootloader/{}", build_type));
    let bootloader = output_dir.join("bootloader");

    // keep debug symbols for bootloader debugging
    let mut cmd = Command::new(&objcopy);
    cmd.arg("--only-keep-debug");
    cmd.arg(&bootloader);
    cmd.arg(&format!("{}.sym", bootloader.to_str().unwrap()));
    let cmd_status = cmd
        .status()
        .expect("Failed to run llvm-objcopy to extract debug symbols");
    if !cmd_status.success() {
        panic!("Failed to extract debug symbols");
    }

    // convert the ELF file into a flat binary
    let mut cmd = Command::new(&objcopy);
    cmd.arg("-I").arg("elf64-x86-64");
    cmd.arg("-O").arg("binary");
    cmd.arg(&bootloader);
    cmd.arg(&format!("{}.bin", bootloader.to_str().unwrap()));
    let cmd_status = cmd
        .status()
        .expect("Failed to run llvm-objcopy to create flat binary");
    if !cmd_status.success() {
        panic!("Failed to create flat binary");
    }

    println!("Done");
}
