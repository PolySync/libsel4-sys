extern crate heck;

use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use heck::ShoutySnakeCase;

/// TODO - docs
/// parse CMakeCache.txt
/// booleans -> features and pub const ...
/// values -> pub const ...
/// include!(concat!(env!("OUT_DIR"), "/<namespace>_config.rs"));
///
/// namespace:
/// - mod <namespace>_config, file <namespace>_config.rs
/// - search for DEP_<NAMESPACE>_CMAKE_CACHE_PATH if cache_path is None
pub fn process_cmake_cache(
    namespace: &str,
    cache_path: Option<PathBuf>,
    output_path: PathBuf,
) {
    let rust_filename = format!("{}_config.rs", namespace);
    let rust_module = format!("{}_config", namespace);
    let cache_env_var = format!(
        "DEP_{}_CMAKE_CACHE_PATH",
        namespace.to_shouty_snake_case()
    );

    let eval_cache_path = cache_path.unwrap_or_else(|| {
        PathBuf::from(env::var(&cache_env_var).expect(&format!(
            "Failed to automatically determine the cache path \
             via the environment variable {}.",
            cache_env_var
        )))
    });

    let in_file =
        BufReader::new(File::open(&*eval_cache_path).expect(&format!(
            "Failed to open the cache file {}",
            eval_cache_path.display()
        )));
    let out_file_path = PathBuf::from(format!(
        "{}/{}",
        output_path.display(),
        rust_filename
    ));

    let out_file = File::create(&out_file_path).unwrap();
    let mut f = BufWriter::new(out_file);

    writeln!(f, "#[allow(dead_code)]").unwrap();
    writeln!(f, "mod {} {{", rust_module).unwrap();

    for line in in_file.lines().filter_map(|result| result.ok()) {
        let line = line.trim();

        // skip comments and empty lines
        if line.starts_with("#") || !line.contains("=") {
            continue;
        }

        // skip these option types
        // - FILEPATH
        // - PATH
        // - INTERNAL
        // - STATIC
        // - UNINITIALIZED
        if line.contains(":FILEPATH=") || line.contains(":PATH=")
            || line.contains(":INTERNAL=")
            || line.contains(":STATIC=")
            || line.contains(":UNINITIALIZED=")
        {
            continue;
        }

        // skip CMAKE options
        if line.starts_with("CMAKE_") {
            continue;
        }

        // TODO - pull in comments/docs for each option
        // they start with '//' C style comment block
        // for now just skip them
        if line.starts_with("//") {
            continue;
        }

        // split line into tokens: key:type=value -> ["key", "type", "value"]
        let tokens: Vec<String> = line.split(|c| c == ':' || c == '=')
            .map(|s| s.to_string())
            .collect();
        let (key, type_hint, value) = (&tokens[0], &tokens[1], &tokens[2]);

        // rustify the key, convert to SCREAMING_SNAKE_CASE
        // ie 'KernelX86IBPBOnContextSwitch' ->
        // 'KERNEL_X86_IBPB_ON_CONTEXT_SWITCH'
        let rusty_key = &key.to_shouty_snake_case();

        assert!(
            rusty_key
                .chars()
                .all(|x| x.is_uppercase() || x.is_digit(10) || x == '_'),
            "Found an invalid key: '{}'",
            rusty_key
        );

        match type_hint.as_ref() {
            "BOOL" => match value.as_ref() {
                "ON" | "TRUE" => {
                    add_bool_as_feature(rusty_key, true);
                    add_bool_as_const(rusty_key, true, &mut f);
                }
                "OFF" | "FALSE" => {
                    add_bool_as_feature(rusty_key, false);
                    add_bool_as_const(rusty_key, false, &mut f);
                }
                _ => add_string_as_const(rusty_key, value, &mut f),
            },
            "STRING" => add_string_as_const(rusty_key, value, &mut f),
            _ => add_string_as_const(rusty_key, value, &mut f),
        }
    }

    writeln!(f, "}}").unwrap();

    println!(
        "cargo:rerun-if-changed={}",
        eval_cache_path.display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        output_path.display()
    );
}

fn add_bool_as_feature(key: &str, enabled: bool) {
    if enabled {
        println!("cargo:rustc-cfg=feature=\"{}\"", key);
    }
}

fn add_bool_as_const(key: &str, enabled: bool, f: &mut BufWriter<File>) {
    writeln!(f, "    pub const {}: bool = {};", key, enabled).unwrap()
}

fn add_string_as_const(key: &str, value: &str, f: &mut BufWriter<File>) {
    match value.parse::<u32>() {
        Ok(_) => {
            writeln!(f, "    pub const {}: usize = {};", key, value).unwrap()
        }
        _ => writeln!(
            f,
            "    pub const {}: &'static str = \"{}\";",
            key, value
        ).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cache_file_processing() {
        ::process_cmake_cache(
            "sel4",
            Some(
                ::env::current_dir()
                    .unwrap()
                    .join("test_data/CMakeCache.txt"),
            ),
            ::env::current_dir().unwrap().join("test_data"),
        );
    }

    #[test]
    fn test_env_cache_file_processing() {
        ::env::set_var(
            "DEP_TEST_CMAKE_CACHE_PATH",
            ::env::current_dir()
                .unwrap()
                .join("test_data/CMakeCache.txt"),
        );
        ::process_cmake_cache(
            "test",
            None,
            ::env::current_dir().unwrap().join("test_data"),
        );
    }
}
