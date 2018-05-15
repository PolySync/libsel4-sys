# libsel4-sys

Builds the sel4 kernel and generates bindings around it,
as configured by a fel4 manifest.

## Project Layout

```
libsel4-sys/
├── Cargo.toml
├── build.rs                <-- Pulls in fel4 toml for CMake configuration
├── CMakeLists.txt
├── deps                    <-- submodules to seL4 repositories
│   ├── musllibc
│   ├── seL4_kernel
│   ├── seL4_libs
│   ├── seL4_tools
│   └── util_libs
├── package
│   └── CMakeLists.txt      <-- custom CMake script to build seL4 artifacts
├── README.md
├── res
│   └── bindgen_wrapper.h
├── src
│   └── lib.rs
└── Xargo.toml
```

## Down-stream Configuration

Down-stream consumers can specify a toml file that contains the CMake configuration
tables via the `FEL4_MANIFEST_PATH` variable.

## Building

Don't forget to run `git submodule update --init` to pull in the seL4 related dependencies
before you start hacking.

This project is intended to be built in the context of the `cargo fel4` command, which manages
the piping of key environment variables relevant to the downstream project.


Builds require that the `FEL4_MANIFEST_PATH` environment variable is set and
includes a path that points to a valid fel4.toml file, as specified by the `fel4-config`
crate.

Additionally, the `RUST_TARGET_PATH` must be supplied, pointing to the directory that
contains the Rust target specification JSON files relevant for the desired build target.

```
RUST_TARGET_PATH=$PWD/test_configs FEL4_MANIFEST_PATH=$PWD/test_configs/fel4.toml xargo rustc --target x86_64-sel4-fel4 -vv
```

## Output Artifacts

If environment variable `FEL4_ARTIFACT_PATH` is set, the kernel and simulation script
will be copied into the directory specified by the variable.
