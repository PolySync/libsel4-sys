# libsel4-sys

## Overview

Builds the sel4 kernel and generates Rust bindings around it,
as configured by a feL4 manifest.

This library provides thin Rust bindings around the [seL4 codebase](https://github.com/seL4/seL4);
more idiomatic Rust wrappers for the enclosed functionality will be supplied in other crates.

Intended for use in projects managed by
[cargo-fel4](https://github.com/PolySync/cargo-fel4), see that repository
for introductory materials.

## Getting Started


```
libsel4-sys/
├── Cargo.toml
├── build.rs                <-- Configures CMake with feL4 manifest data, runs CMake and bindgen
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

### Dependencies

libsel4-sys uses git submodules to make seL4 related code available locally.

The few Rust dependencies are managed by Cargo.toml, so a Cargo is necessary, as well
as Xargo for cross-compilation. Rustup is the recommended Rust toolchain manager.

```
# Install rustup
curl -f -L https://static.rust-lang.org/rustup.sh -O
sh rustup.sh
# The nightly toolchain is currently required
rustup install nightly

# Install Xargo, which requires rust-src component
rustup component add rust-src
cargo install xargo
```

The seL4 build system we're wrapping has non-trivial dependencies on CMake and Ninja.

### CMake

CMake version `3.7.2` or greater is required.

Binary releases are available from [cmake.org](https://cmake.org/download/).

### Ninja

Ninja version `1.7.1` or greater is required.

Binary releases are available from [github](https://github.com/ninja-build/ninja/releases).

### Cross compiler toolchains

```
$ sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
```

### Building

This project is intended to be built in the context of the `cargo fel4` command, which manages
the piping of key environment variables relevant to the downstream project.

See the [cargo-fel4 repository](https://github.com/PolySync/cargo-fel4) for more direction, but the general idea is to create
a Rust project with `libsel4-sys` as a dependency, add a fel4.toml file, and to run `cargo fel4 build`.

While not recommended for general use, manual builds are possible.

Don't forget to run `git submodule update --init` to pull in the seL4 related dependencies
to the local filesystem before attempting a build.

Builds require that the `FEL4_MANIFEST_PATH` environment variable is set and
includes a path that points to a valid fel4.toml file, as specified by the `fel4-config`
crate.

Additionally, the `RUST_TARGET_PATH` must be supplied, pointing to the directory that
contains the Rust target specification JSON files relevant for the desired build target.

```
RUST_TARGET_PATH=$PWD/test_configs FEL4_MANIFEST_PATH=$PWD/test_configs/fel4.toml xargo rustc --target x86_64-sel4-fel4 -vv
```

If environment variable `FEL4_ARTIFACT_PATH` is set, the kernel and simulation script
will be copied into the directory specified by the variable.

## Usage

### Examples

### API

The generated bindings should be treated as relatively ephemeral and dynamic compared
to most Rust libraries. The output is context-specific to the target (e.g. "armv7-sel4-fel4")
and the set of configuration flags derived from the input feL4 manifest file.

See the Rust docs for a surface-level overview of the raw APIs exposed.

## Tests

Currently, all testing is done one level up, in the `cargo-fel4` repo,
which has the capability to auto-generate the appropriate test runner
code and exercise the resultant artifacts.

### Test Dependencies

See the [cargo-fel4 repository](https://github.com/PolySync/cargo-fel4).

### Running Tests

`cargo fel4 new tests && cd tests && cargo fel4 test build && cargo fel4 test simulate`

# License

This project is released under the MIT license. See the [dependencies README](deps/README.md)
for more details.
