//! This is the build script for the `libsel4-sys` package.
//!
//! It builds both the kernel binary and libsel4.a C bindings.
//!

extern crate bindgen;
extern crate cmake;
extern crate fel4_config;

use bindgen::Builder;
use cmake::Config as CmakeConfig;
use fel4_config::{Fel4Config, SupportedPlatform, SupportedTarget};
use std::env;
use std::fs;
use std::path::PathBuf;


struct BindgenHeaderIncludeConfig {
    kernel_arch: String,
    kernel_sel4_arch: String,
    width: String,
    platform: String,
}

fn main() {
    // Resolve fel4 configuration from the manifest located via FEL4_MANIFEST_PATH and PROFILE env-vars
    let (fel4_manifest_path, build_profile ) = match fel4_config::infer_manifest_location_from_env() {
        Ok(p) => p,
        Err(e) => panic!("libsel4-sys build.rs had trouble figuring out where to pull fel4 config from. {}", e),
    };
    println!("cargo:rerun-if-changed={}", fel4_manifest_path.display());
    let fel4 = match fel4_config::get_fel4_config(&fel4_manifest_path, &build_profile) {
        Ok(f) => f,
        Err(e) => { panic!("libsel4-sys build.rs ran into trouble with the fel4 manifest found at {:?}. {}", &fel4_manifest_path, e) },
    };
    println!("cargo:rerun-if-changed={}", fs::canonicalize(&fel4_manifest_path).expect("Could not canonicalize the fel4 manifest path").display());

    // Configure the CMake build using the data resolved from the fel4 manifest
    let mut cmake_build_config = CmakeConfig::new(".");
    match fel4_config::configure_cmake_build_from_env(&mut cmake_build_config, &fel4) {
        Ok(_) => {},
        Err(e) => { panic!("libsel4-sys build.rs ran into trouble configuring the sel4 kernel CMake build when using the fel4 manifest from {:?}. {}", &fel4_manifest_path, e) },
    };

    // delete any existing CMakeCache.txt to prevent seL4/CMake from
    // unexpected reconfigurations
    let prev_cache_path = PathBuf::from(getenv_unwrap("OUT_DIR"))
        .join("build")
        .join("CMakeCache.txt");

    if prev_cache_path.exists() {
        fs::remove_file(prev_cache_path)
            .expect("failed to delete previous CMakeCache.txt file");
    }

    // perform the cmake build
    let cargo_output_path = cmake_build_config.build();

    generate_bindings(
        &fel4,
        cargo_output_path.join("build").join("staging"),
    );

    // links = "sel4", these non-cargo variables can be read by consumer
    // packages
    print_cargo_links_keys(cargo_output_path.clone());

    print_cargo_rerun_if_flags();

    // copy artifacts if environment variable is set
    let dest_env = env::var("FEL4_ARTIFACT_PATH");
    match dest_env {
        Ok(p) => copy_artifacts(cargo_output_path.clone(), PathBuf::from(p)),
        Err(_) => (),
    }

    println!(
        "cargo:rustc-link-search=native={}",
        cargo_output_path.display()
    );

    // native libsel4.a location
    println!(
        "cargo:rustc-link-search=native={}",
        cargo_output_path.join("build").join("libsel4").display()
    );

    println!("cargo:rustc-link-lib=static=sel4");
}

/// Print common links keys used by consumer packages.
///
/// You can access these as environment variables:
/// - `DEP_SEL4_CMAKE_CACHE_PATH`
/// - `DEP_SEL4_KERNEL_PATH`
/// - `DEP_SEL4_SIMULATION_SCRIPT_PATH`
fn print_cargo_links_keys(cargo_output_path: PathBuf) {
    println!(
        "cargo:cmake_cache_path={}",
        cargo_output_path
            .join("build")
            .join("CMakeCache.txt")
            .display()
    );

    println!(
        "cargo:kernel_path={}",
        cargo_output_path
            .join("build")
            .join("images")
            .join("kernel")
            .display()
    );

    println!(
        "cargo:simulation_script_path={}",
        cargo_output_path.join("build").join("simulate").display()
    );
}

/// Print common environment rerun-if's.
fn print_cargo_rerun_if_flags() {
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=FEL4_MANIFEST_PATH");
    println!("cargo:rerun-if-env-changed=FEL4_ARTIFACT_PATH");
    println!("cargo:rerun-if-env-changed=FEL4_ROOT_TASK_IMAGE_PATH");
    println!("cargo:rerun-if-changed=package");
    println!("cargo:rerun-if-changed=package/CMakeLists.txt");
}

/// Copy build external build artifacts (kernel/simulation-script) into the
/// artifact directory.
fn copy_artifacts(artifact_path: PathBuf, output_path: PathBuf) {
    if !output_path.exists() {
        fs::create_dir_all(&output_path).unwrap();
    }

    fs::copy(
        artifact_path.join("build").join("images").join("kernel"),
        output_path.join("kernel"),
    ).unwrap();

    fs::copy(
        artifact_path.join("build").join("simulate"),
        output_path.join("simulate"),
    ).unwrap();

    fs::copy(
        artifact_path.join("build").join("CMakeCache.txt"),
        output_path.join("CMakeCache.txt"),
    ).unwrap();

    println!(
        "cargo:rerun-if-changed={}",
        output_path.join("kernel").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        output_path.join("simulate").display()
    );
}

/// Generate the libsel4 Rust bindings.
fn generate_bindings(fel4: &Fel4Config, include_path: PathBuf) {
    let bindgen_include_config = get_bindgen_include_config(fel4);

    let target_args = if fel4.target == SupportedTarget::Armv7Sel4Fel4 {
        String::from("-mfloat-abi=hard")
    } else {
        String::from("")
    };

    let bindings = Builder::default()
        .header("res/bindgen_wrapper.h")
        .whitelist_recursively(true)
        .no_copy("*")
        .use_core()
        // our custom c_types
        .ctypes_prefix("c_types")
        // TODO - verify that we should be implementing these in src/lib.rs
        .blacklist_type("strcpy")
        .blacklist_type("__assert_fail")
        .clang_arg(target_args)
        .clang_arg(format!("-I{}", include_path.join("include").display()))
        .clang_arg(format!("-I{}", include_path.join(bindgen_include_config.kernel_arch).display()))
        .clang_arg(format!("-I{}", include_path.join(bindgen_include_config.kernel_sel4_arch).display()))
        .clang_arg(format!("-I{}", include_path.join(bindgen_include_config.width).display()))
        .clang_arg(format!("-I{}", include_path.join(bindgen_include_config.platform).display()))
        .clang_arg(format!("-I{}", include_path.join("autoconf").display()))
        .clang_arg(format!("-I{}", include_path.join("gen_config").display()))
        .clang_arg(format!("-I{}", include_path.join("include").display()))
        .generate()
        .expect("failed to generate bindings");

    bindings
        .write_to_file(
            PathBuf::from(getenv_unwrap("OUT_DIR")).join("bindings.rs"),
        )
        .expect("failed to write bindings to file");
}

/// Parses the target and platform data to produce
/// bindgen compatable include
/// token Strings.
///
/// Returns a BindgenHeaderIncludeConfig.
fn get_bindgen_include_config(
    fel4: &Fel4Config,
) -> BindgenHeaderIncludeConfig {
    // TODO - expand here when we add more supported platforms/targets in fel4-config
    // or move more of this include knowledge to fel4-config (painful)
    match &fel4.target {
        &SupportedTarget::X8664Sel4Fel4 => {
            BindgenHeaderIncludeConfig {
                kernel_arch: String::from("x86"),
                kernel_sel4_arch: String::from("x86_64"),
                width: String::from("64"),
                platform: fel4.platform.full_name().to_string(),
            }
        },
        t @ &SupportedTarget::Armv7Sel4Fel4 => {
            // TODO - add more mappings as platform options expand
            //"exynos5410" => "exynos5",
            //"exynos5422" => "exynos5",
            //"exynos5250" => "exynos5",
            //"imx7sabre" => "imx7",
            //"rpi3" => "bcm2837",

            // Platform names don't always match the associated sub-directory used for header includes
            // so a mapping is necessary
            let plat_include_dir = match &fel4.platform {
                p @ &SupportedPlatform::PC99 => {
                    panic!("{} target is not supported in combination with {} platform", t.full_name(), p.full_name())
                },
                &SupportedPlatform::Sabre => { "imx6"},
            };
            BindgenHeaderIncludeConfig {
                kernel_arch: String::from("arm"),
                kernel_sel4_arch: String::from("aarch32"),
                width: String::from("32"),
                platform: plat_include_dir.to_string(),
            }
        },
    }
}



/// Return an environment variable as a String.
fn getenv_unwrap(v: &str) -> String {
    match env::var(v) {
        Ok(s) => s,
        Err(..) => fail(&format!("environment variable `{}` not defined", v)),
    }
}

/// Failure with panic.
fn fail(s: &str) -> ! {
    panic!("\n{}\n\nlibsel4-sys build script failed", s)
}
