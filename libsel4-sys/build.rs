//! This is the build script for the `libsel4-sys` package.
//!
//! It builds both the kernel binary and libsel4.a C bindings.
//!

extern crate bindgen;
extern crate cmake;
extern crate toml;

use bindgen::Builder;
use cmake::Config as CmakeConfig;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(PartialEq)]
pub enum ArchHint {
    X86,
    ARM,
    ARMV8,
}

struct TomlConfig {
    target: String,
    target_arch: ArchHint,
    platform: String,
    cmake_target_config: toml::Value,
    cmake_profile_config: toml::Value,
    cmake_platform_config: toml::Value,
}

struct BindgenHeaderIncludeConfig {
    kernel_arch: String,
    kernel_sel4_arch: String,
    width: String,
    platform: String,
}

fn main() {
    let mut cmake_build_config = CmakeConfig::new(".");

    // configure the CMake build from fel4.toml
    let toml_config = configure_cmake_build(&mut cmake_build_config);

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
        &toml_config,
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
        fs::create_dir(&output_path).unwrap();
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
fn generate_bindings(toml_config: &TomlConfig, include_path: PathBuf) {
    let bindgen_include_config = get_bindgen_include_config(toml_config);

    let target_args = if toml_config.target_arch == ArchHint::ARM {
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
    toml_config: &TomlConfig,
) -> BindgenHeaderIncludeConfig {
    if toml_config.target_arch == ArchHint::X86 {
        BindgenHeaderIncludeConfig {
            kernel_arch: String::from("x86"),
            kernel_sel4_arch: String::from("x86_64"),
            width: String::from("64"),
            platform: toml_config.platform.to_string(),
        }
    } else if toml_config.target_arch == ArchHint::ARM {
        // some platform names don't map one-to-one
        let plat_include = match toml_config.platform.as_str() {
            "sabre" => "imx6",
            "exynos5410" => "exynos5",
            "exynos5422" => "exynos5",
            "exynos5250" => "exynos5",
            "imx7sabre" => "imx7",
            "rpi3" => "bcm2837",
            p => p,
        };

        BindgenHeaderIncludeConfig {
            kernel_arch: String::from("arm"),
            kernel_sel4_arch: String::from("aarch32"),
            width: String::from("32"),
            platform: plat_include.to_string(),
        }
    } else if toml_config.target_arch == ArchHint::ARMV8 {
        BindgenHeaderIncludeConfig {
            kernel_arch: String::from("arm"),
            kernel_sel4_arch: String::from("aarch64"),
            width: String::from("64"),
            platform: toml_config.platform.to_string(),
        }
    } else {
        fail(&format!("unsupported target '{}'", toml_config.target))
    }
}

/// Configure a CMake build configuration from toml.
///
/// Returns a TomlConfig representation of fel4.toml.
fn configure_cmake_build(cmake_config: &mut CmakeConfig) -> TomlConfig {
    let cargo_target = getenv_unwrap("TARGET");

    let root_dir = getenv_unwrap("CARGO_MANIFEST_DIR");

    let root_path = Path::new(&root_dir);

    let kernel_path = root_path.join("deps").join("seL4_kernel");

    let fel4_manifest = PathBuf::from(getenv_unwrap("FEL4_MANIFEST_PATH"));

    println!("cargo:rerun-if-changed={}", fel4_manifest.display());

    // parse fel4.toml
    let toml_config = get_toml_config(fel4_manifest, &getenv_unwrap("PROFILE"));

    if cargo_target != toml_config.target {
        fail(&format!("Cargo is attempting to build for the {} target, however fel4.toml has declared the target to be {}", cargo_target, toml_config.target));
    }

    // CMAKE_TOOLCHAIN_FILE is resolved immediately by CMake
    cmake_config.define("CMAKE_TOOLCHAIN_FILE", kernel_path.join("gcc.cmake"));

    cmake_config.define("KERNEL_PATH", kernel_path);

    // add options from build profile sub-table
    add_cmake_options_from_table(
        &toml_config.cmake_profile_config,
        cmake_config,
    );

    // add options from target sub-table
    add_cmake_options_from_table(
        &toml_config.cmake_target_config,
        cmake_config,
    );

    // add options from platform sub-table
    add_cmake_options_from_table(
        &toml_config.cmake_platform_config,
        cmake_config,
    );

    // seL4 handles these so we clear them to prevent cmake-rs from
    // auto-populating
    cmake_config.define("CMAKE_C_FLAGS", "");
    cmake_config.define("CMAKE_CXX_FLAGS", "");

    // Ninja generator
    cmake_config.generator("Ninja");

    toml_config
}

/// Add CMake configurations from a toml table.
fn add_cmake_options_from_table(
    toml_table: &toml::Value,
    cmake_config: &mut CmakeConfig,
) {
    for (key, value) in toml_table.as_table().unwrap() {
        // ignore other tables within this one
        if value.is_table() {
            continue;
        }

        add_cmake_definition(key, value, cmake_config);
    }
}

/// Add a CMake configuration definition
fn add_cmake_definition(
    key: &String,
    value: &toml::Value,
    config: &mut CmakeConfig,
) {
    // booleans use the :<type> syntax, with ON/OFF values
    // everything else is treated as a string
    if value.type_str() == "boolean" {
        if value.as_bool().unwrap() == true {
            config.define(format!("{}:BOOL", key), "ON");
        } else {
            config.define(format!("{}:BOOL", key), "OFF");
        }
    } else if value.type_str() == "integer" {
        config.define(
            key,
            value
                .as_integer()
                .expect(&format!(
                    "failed to convert key '{}' to integer",
                    value
                ))
                .to_string(),
        );
    } else {
        config.define(
            key,
            value
                .as_str()
                .expect(&format!("failed to convert key '{}' to str", value)),
        );
    }
}

/// Returns a TomlConfig generated from a fel4.toml file.
fn get_toml_config(path: PathBuf, build_profile: &String) -> TomlConfig {
    let mut manifest_file = File::open(&path)
        .expect(&format!("failed to open manifest file {}", path.display()));

    let mut contents = String::new();

    manifest_file.read_to_string(&mut contents).unwrap();

    let manifest = contents
        .parse::<toml::Value>()
        .expect("failed to parse fel4.toml");

    let fel4_table = match manifest.get("fel4") {
        Some(t) => t,
        None => fail("fel4.toml is missing fel4 table"),
    };

    let target = match fel4_table.get("target") {
        Some(t) => String::from(t.as_str().unwrap()),
        None => fail("fel4.toml is missing target key"),
    };

    let platform = match fel4_table.get("platform") {
        Some(t) => String::from(t.as_str().unwrap()),
        None => fail("fel4.toml is missing platform key"),
    };

    let target_config = match manifest.get(&target) {
        Some(t) => t,
        None => fail("fel4.toml is missing the target table"),
    };

    TomlConfig {
        target: target.clone(),
        target_arch: if target.contains("arm") {
            ArchHint::ARM
        } else if target.contains("aarch64") {
            ArchHint::ARMV8
        } else if target.contains("x86") {
            ArchHint::X86
        } else {
            fail("fel4.toml target is not supported");
        },
        platform: platform.clone(),
        cmake_target_config: target_config.clone(),
        cmake_profile_config: match target_config.get(&build_profile) {
            Some(t) => t.clone(),
            None => fail("fel4.toml is missing build profile table"),
        },
        cmake_platform_config: match target_config.get(&platform) {
            Some(t) => t.clone(),
            None => fail("fel4.toml is missing target platform table"),
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
