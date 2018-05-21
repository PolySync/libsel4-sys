# libsel4-sys

Builds the sel4 kernel and generates Rust bindings around it,
as configured by a fel4 manifest.

This library provides thin Rust bindings around the [seL4 codebase](https://github.com/seL4/seL4);
more idiomatic Rust wrappers for the enclosed functionality will be supplied in other crates.

Intended for use in projects managed by
[cargo fel4](https://github.com/PolySync/cargo-fel4), see that repository
for introductory materials.

## Project Layout

```
libsel4-sys/
├── Cargo.toml
├── build.rs                <-- Configures CMake with fel4 manifest data, runs CMake and bindgen
├── CMakeLists.txt
├── deps                    <-- submodules to seL4 repositories
│   ├── musllibc
│   ├── seL4_kernel
│   ├── seL4_libs
│   ├── seL4_tools
│   └── util_libs
├── package
│   └── CMakeLists.txt      <-- custom CMake script wrapper to build seL4 artifacts
├── README.md
├── res
│   └── bindgen_wrapper.h
├── src
│   └── lib.rs              <-- thin wrapper around the generated bindings
└── Xargo.toml
```

## Building

Don't forget to run `git submodule update --init` to pull in the seL4 related dependencies
to the local filesystem before attempting a build.

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

The generated bindings should be treated as relatively ephemeral and dynamic compared
to most Rust libraries. The output is context-specific to the target (e.g. "arm-sel4-fel4")
and the set of configuration
flags derived from the input fel4 manifest file.

If environment variable `FEL4_ARTIFACT_PATH` is set, the kernel and simulation script
will be copied into the directory specified by the variable.

## License

This project is released under the MIT license. See the [dependencies README](deps/README.md)
for more details.
