use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let rustc = Command::new(env::var_os("RUSTC").expect("Cargo must provide RUSTC"))
        .arg("--version")
        .output()
        .expect("read rustc version");
    assert!(rustc.status.success(), "rustc --version failed");
    println!(
        "cargo:rustc-env=BUILD_RUSTC={}",
        String::from_utf8(rustc.stdout)
            .expect("rustc version must be UTF-8")
            .trim()
    );
    for name in ["TARGET", "PROFILE"] {
        println!("cargo:rustc-env=BUILD_{name}={}", env::var(name).unwrap());
    }
}
