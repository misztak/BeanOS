/*!
A glorified build script for the OS and bootloader.

*/

use llvm_tools_build;
use std::{env, path::PathBuf, process::Command};

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

    println!(
        "\n### Step 1: [Building BeanOS in {} mode] ###\n",
        build_type
    );

    let project_root_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Environment variable 'CARGO_MANIFEST_DIR' undefined");
    let kernel_dir = PathBuf::from(&project_root_dir).join("os");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&kernel_dir);
    cmd.arg("--color=always");
    cmd.arg("build");
    if is_release_build {
        cmd.arg("--release");
    }
    let kernel_cmd_output = cmd.status().expect("Failed to run cargo to build kernel");

    //let kernel_output = std::str::from_utf8(&output.stderr).unwrap().trim_end();
    //if kernel_output.contains("dev") {
    //    println!("{}", kernel_output);
    //}

    assert!(kernel_cmd_output.success(), "XXXX --- Failed to build kernel --- XXXX");

    //
    // Step 2: Build the bootloader
    //

    println!(
        "\n### Step 2: [Building bootloader in {} mode] ###\n",
        build_type
    );

    let bootloader_dir = PathBuf::from(&project_root_dir).join("bootloader");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&bootloader_dir);
    cmd.arg("build");
    if is_release_build {
        cmd.arg("--release");
    }
    let cmd_status = cmd
        .status()
        .expect("Failed to run cargo to build bootloader");
    assert!(cmd_status.success(), "XXXX --- Failed to build bootloader --- XXXX");

    //
    // Step 3: Convert ELF file into flat binary
    //

    println!("\n### Step 3: [Creating flat binary] ###\n");

    let output_dir =
        bootloader_dir.join(format!("target/x86_64-bean_os_bootloader/{}", build_type));
    let bootloader_elf = output_dir.join("bootloader");

    let mut bootloader_image = PathBuf::from(&bootloader_elf);
    bootloader_image.set_extension("bin");

    let mut bootloader_symbols = PathBuf::from(&bootloader_elf);
    bootloader_symbols.set_extension("sym");

    // keep debug symbols for bootloader debugging
    let mut cmd = Command::new(&objcopy);
    cmd.arg("--only-keep-debug");
    cmd.arg(&bootloader_elf);
    cmd.arg(&bootloader_symbols);
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
    cmd.arg(&bootloader_elf);
    cmd.arg(&bootloader_image);
    let cmd_status = cmd
        .status()
        .expect("Failed to run llvm-objcopy to create flat binary");
    if !cmd_status.success() {
        panic!("Failed to create flat binary");
    }

    // check the filesize of the bootloader and kernel inside the final ELF file
    // only available on linux right now because it calls `readelf`
    if cfg!(target_os = "linux") {
        check_filesize(&bootloader_elf);
    }

    println!("### DONE! ###");
}

#[cfg(target_os = "linux")]
fn check_filesize(elf_path: &PathBuf) {
    let mut cmd = Command::new("readelf");
    cmd.arg("-t");
    cmd.arg(&elf_path);
    let output = cmd.output().expect("Failed to run readelf");
    assert!(output.status.success(), "Readelf command failed");

    let seg_info: Vec<&str> = std::str::from_utf8(&output.stdout)
        .expect("Failed to parse readelf output")
        .split('\n')
        .collect();

    let mut bootloader_size: u64 = 0;
    let mut kernel_size: u64 = 0;

    // yikes
    for (i, &line) in seg_info.iter().enumerate() {
        if line.contains(".bootloader") {
            let size_str = seg_info[i + 2].trim().split_once(' ').unwrap().0;
            bootloader_size = u64::from_str_radix(size_str, 16).unwrap();
        }
        if line.contains(".kernel") {
            let size_str = seg_info[i + 2].trim().split_once(' ').unwrap().0;
            kernel_size = u64::from_str_radix(size_str, 16).unwrap();
        }
    }

    println!("Bootloader segment size: 0x{:x} ({} out of 480 KiB used)", bootloader_size, bootloader_size / 1024);
    println!("Kernel segment size:     0x{:x} ({} KiB)", kernel_size, kernel_size / 1024);

    if bootloader_size > 480 * 1024 {
        eprintln!("\x1b[93mWARNING: Bootloader might be overflowing usable memory region!\x1b[0m");
    }

    println!();

}
