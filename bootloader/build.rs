use std::{env, process::{self, Command}, path::PathBuf};
use llvm_tools_build;

fn main() {
    // rebuild if one of these files was modified
    println!("cargo:rerun-if-changed=linker.ld");

    // output directory (build script should not modify any files outside this directory)
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR undefined"));

    // get llvm_tools (requires llvm-tools-preview)
    let llvm_tools = match llvm_tools_build::LlvmTools::new() {
        Ok(tools) => tools,
        Err(llvm_tools_build::Error::NotFound) => {
            eprintln!("llvm_tools not found. Install it through: 'rustup component add llvm-tools-preview'");
            process::exit(1);
        }
        Err(err) => {
            eprintln!("Failed to load llvm_tools: {:?}", err);
            process::exit(1);
        }
    };

    let objcopy = llvm_tools
        .tool(&llvm_tools_build::exe("llvm-objcopy"))
        .expect("llvm-objcopy not found");
    let ar = llvm_tools
        .tool(&llvm_tools_build::exe("llvm-ar"))
        .expect("llvm-ar not found");

    // find the kernel file with the same profile (i.e. debug or release build)
    let profile = env::var("PROFILE").expect("PROFILE undefined");
    let bootloader_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR undefined");
    let kernel_dir = PathBuf::from(&bootloader_dir)
        .join(format!("../os/target/x86_64-bean_os/{}/", profile))
        .canonicalize().expect("Failed to canonicalize kernel path");

    let kernel = kernel_dir.join("bean_os");

    // strip kernel binary
    let kernel_stripped = out_dir.join("bean_os-stripped");
    let mut cmd = Command::new(&objcopy);
    cmd.arg("--strip-debug");
    cmd.arg(&kernel);
    cmd.arg(&kernel_stripped);
    let cmd_status = cmd
        .status()
        .expect("Failed to run llvm-objcopy to strip debug symbols");
    if !cmd_status.success() {
        eprintln!("Failed to strip debug symbols");
        process::exit(1);
    }

    // wrap the kernel ELF as a binary blob in a new ELF file
    let kernel_bin = out_dir.join("bean_os.o");
    let kernel_stripped_only_underscores = kernel_stripped.to_str().expect("Invalid path")
        .replace('.', "_")
        .replace('/', "_")
        .replace('-', "_");

    let mut cmd = Command::new(&objcopy);
    cmd.current_dir(&out_dir);
    cmd.arg("-I").arg("binary");
    cmd.arg("-O").arg("elf64-x86-64");
    cmd.arg("--binary-architecture=i386:x86-64");
    cmd.arg("--rename-section").arg(".data=.kernel");
    cmd.arg("--redefine-sym").arg(format!("_binary_{}_start=_kernel_start_addr", kernel_stripped_only_underscores));
    cmd.arg("--redefine-sym").arg(format!("_binary_{}_end=_kernel_end_addr", kernel_stripped_only_underscores));
    cmd.arg("--redefine-sym").arg(format!("_binary_{}_size=_kernel_size", kernel_stripped_only_underscores));
    cmd.arg(&kernel_stripped);
    cmd.arg(&kernel_bin);
    let cmd_status = cmd
        .status()
        .expect("Failed to run llvm-objcopy to wrap kernel blob");
    if !cmd_status.success() {
        eprintln!("Failed to wrap kernel blob");
        process::exit(1);
    }

    // create an archive for linking
    let kernel_archive = out_dir.join("libbean_os.a");

    let mut cmd = Command::new(&ar);
    cmd.arg("crs");
    cmd.arg(&kernel_archive);
    cmd.arg(&kernel_bin);
    let cmd_status = cmd
        .status()
        .expect("Failed to run llvm-ar to create archive");
    if !cmd_status.success() {
        eprintln!("Failed to create archive");
        process::exit(1);
    }

    // link kernel blob with bootloader
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=bean_os");
}
