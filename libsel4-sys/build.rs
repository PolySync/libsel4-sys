extern crate bindgen;
extern crate cmake;
extern crate package_config;
extern crate toml;

use bindgen::Builder;
use cmake::Config;
use package_config::process_cmake_cache;
use std::env;
use std::fs::File;
use std::fs::{copy, create_dir, remove_file};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use toml::Value;

fn main() {
    let mut config = Config::new(".");

    configure_cmake_build(&mut config);

    // delete any existing CMakeCache.txt to prevent seL4/CMake from
    // unexpected reconfigurations
    let prev_cache_path = PathBuf::from(getenv_unwrap("OUT_DIR"))
        .join("build")
        .join("CMakeCache.txt");

    if prev_cache_path.exists() {
        remove_file(prev_cache_path)
            .expect("failed to delete previous CMakeCache.txt file");
    }

    let cargo_output_path = config.build();

    process_cmake_cache(
        "sel4",
        Some(
            cargo_output_path
                .join("build")
                .join("CMakeCache.txt"),
        ),
        PathBuf::from(getenv_unwrap("OUT_DIR")),
    );

    generate_bindings(cargo_output_path.join("build").join("staging"));

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
        cargo_output_path
            .join("build")
            .join("libsel4")
            .display()
    );

    println!("cargo:rustc-link-lib=static=sel4");
}

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
        cargo_output_path
            .join("build")
            .join("simulate")
            .display()
    );
}

fn print_cargo_rerun_if_flags() {
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rerun-if-env-changed=FEL4_MANIFEST_PATH");
    println!("cargo:rerun-if-env-changed=FEL4_ARTIFACT_PATH");
    println!("cargo:rerun-if-changed=package");
    println!("cargo:rerun-if-changed=package/CMakeLists.txt");
}

fn copy_artifacts(artifact_path: PathBuf, output_path: PathBuf) {
    if !output_path.exists() {
        create_dir(&output_path).unwrap();
    }

    copy(
        artifact_path
            .join("build")
            .join("images")
            .join("kernel"),
        output_path.join("kernel"),
    ).unwrap();

    copy(
        artifact_path.join("build").join("simulate"),
        output_path.join("simulate"),
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

fn generate_bindings(include_path: PathBuf) {
    let target = getenv_unwrap("TARGET");

    let (kernel_arch, sel4_arch, width, plat) =
        get_bindgen_target_include_dirs(&target);

    let bindings = Builder::default()
        .header("res/bindgen_wrapper.h")
        .whitelist_recursively(true)
        .no_copy("*")
        .use_core()
        .ctypes_prefix("c_types")
        // we implement these
        .blacklist_type("strcpy")
        .blacklist_type("__assert_fail")
        // TODO - do we need this level of configuration?
        //.clang_arg("-mfloat-abi=hard")
        .clang_arg(format!("-I{}", include_path.join("include").display()))
        .clang_arg(format!("-I{}", include_path.join(kernel_arch).display()))
        .clang_arg(format!("-I{}", include_path.join(sel4_arch).display()))
        .clang_arg(format!("-I{}", include_path.join(width).display()))
        .clang_arg(format!("-I{}", include_path.join(plat).display()))
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

/// returns kernel_arch, kernel_sel4_arch, width, platform
fn get_bindgen_target_include_dirs(
    target: &String,
) -> (String, String, String, String) {
    if target == "x86_64-sel4-helios" {
        (
            "x86".to_string(),
            "x86_64".to_string(),
            "64".to_string(),
            "pc99".to_string(),
        )
    } else if target == "arm-sel4-helios" {
        // TODO
        fail(&format!(
            "target '{}' has platform hard coded",
            target
        ));
    /*
        (
            "arm".to_string(),
            "aarch32".to_string(),
            "32".to_string(),
            "imx6".to_string(),
        )
        */
    } else {
        fail(&format!("unsupported target '{}'", target))
    }
}

fn configure_cmake_build(config: &mut Config) {
    let root_dir = getenv_unwrap("CARGO_MANIFEST_DIR");

    let root_path = Path::new(&root_dir);

    let kernel_path = root_path.join("deps").join("seL4_kernel");

    let fel4_manifest = match env::var("FEL4_MANIFEST_PATH") {
        Ok(v) => PathBuf::from(v),
        Err(_) => root_path.join("fel4.toml"),
    };

    println!(
        "cargo:rerun-if-changed={}",
        fel4_manifest.display()
    );

    let cmake_options = get_cmake_options_table(fel4_manifest);

    // CMAKE_TOOLCHAIN_FILE is resolved immediately by CMake
    config.define(
        "CMAKE_TOOLCHAIN_FILE",
        kernel_path.join("gcc.cmake"),
    );

    config.define("KERNEL_PATH", kernel_path);

    // add options inferred from target specification
    add_cmake_target_options(&cmake_options, config);

    // seL4 handles these so we clear them to prevent cmake-rs from
    // auto-populating
    config.define("CMAKE_C_FLAGS", "");
    config.define("CMAKE_CXX_FLAGS", "");

    for (key, value) in cmake_options
        .as_table()
        .expect("failed to read cmake options Cargo.toml table")
    {
        // ignore other tables within this one
        if value.is_table() {
            continue;
        }

        add_cmake_definition(key, value, config);
    }

    // Ninja generator
    config.generator("Ninja");
}

fn add_cmake_target_options(options: &toml::Value, config: &mut Config) {
    let target = getenv_unwrap("TARGET");

    let target_table = match options.get(&target) {
        Some(ht) => match ht {
            Value::Table(h) => h,
            _ => fail(&format!(
                "target '{}' section is malformed",
                &target
            )),
        },
        None => fail(&format!(
            "target '{}' section is missing",
            &target
        )),
    };

    for (key, value) in target_table {
        // ignore other tables within this one
        if value.is_table() {
            continue;
        }

        add_cmake_definition(key, value, config);
    }
}

fn add_cmake_definition(
    key: &String,
    value: &toml::Value,
    config: &mut Config,
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
            value.as_str().expect(&format!(
                "failed to convert key '{}' to str",
                value
            )),
        );
    }
}

fn get_cmake_options_table(path: PathBuf) -> toml::Value {
    let mut manifest_file = File::open(&path).expect(&format!(
        "failed to open manifest file {}",
        path.display()
    ));

    let mut contents = String::new();

    manifest_file
        .read_to_string(&mut contents)
        .unwrap();

    let manifest = contents.parse::<toml::Value>().unwrap();
    match manifest.get("sel4-cmake-options") {
        Some(t) => t.clone(),
        None => panic!("missing sel4-cmake-options"),
    }
}

fn getenv_unwrap(v: &str) -> String {
    match env::var(v) {
        Ok(s) => s,
        Err(..) => fail(&format!(
            "environment variable `{}` not defined",
            v
        )),
    }
}

fn fail(s: &str) -> ! {
    panic!("\n{}\n\nbuild script failed", s)
}
