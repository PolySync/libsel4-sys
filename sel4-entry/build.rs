#[macro_use]
extern crate maplit;

use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let arch_to_target = hashmap! {
        "x86" => "i686-unknown-linux-gnu",
        "x86_64" => "x86_64-unknown-linux-gnu",
        "arm" => "arm-linux-gnueabihf",
    };
    for (arch, llvmtriple) in &arch_to_target {
        assert!(
            Command::new("/usr/bin/env")
                .arg("clang")
                .arg("-fPIC")
                .arg(&*format!("src/asm/{}.s", arch))
                .args(&[
                    "-c",
                    "-target",
                    llvmtriple,
                    "-o",
                    &*format!("{}/{}.o", out_dir, arch),
                ])
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("/usr/bin/env")
                .arg("ar")
                .arg("crus")
                .arg(format!("{}/lib{}.a", out_dir, arch))
                .arg(&*format!("{}/{}.o", out_dir, arch))
                .status()
                .unwrap()
                .success()
        );
    }

    println!("cargo:rustc-link-search=native={}", out_dir);

    let target = env::var("TARGET").unwrap();
    if target == "i686-sel4-fel4" {
        println!("cargo:rustc-link-lib=static=x86");
    } else if target == "x86_64-sel4-fel4" {
        println!("cargo:rustc-link-lib=static=x86_64");
    } else if target == "arm-sel4-fel4" {
        println!("cargo:rustc-link-lib=static=arm");
    }
}
