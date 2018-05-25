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

### Dependencies

libsel4-sys uses git submodules to make seL4 related code available
locally. Building the seL4 code requires several system dependencies
to be installed. The few Rust dependencies are managed by Cargo.toml,
so Cargo is necessary, as well as Xargo for cross-compilation. Rustup
is the recommended Rust toolchain manager.

* [Rust Nightly](https://github.com/rust-lang-nursery/rustup.rs)
  ```bash
  # Download the rustup install script
  wget https://static.rust-lang.org/rustup/rustup-init.sh
  
  # Install rustup
  chmod +x rustup-init.sh
  sh rustup-init.sh
  
  rustup install nightly
  ```
* [xargo](https://github.com/japaric/xargo)
  ```bash
  # Xargo requires rust-src component
  rustup component add rust-src
  
  # Install Xargo
  cargo install xargo
  ```
* [Cross Compiler Toolchains](https://gcc.gnu.org/)
  ```bash
  # Used by the armv7-sel4-fel4 target
  sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
  
  # Used by the aarch64-sel4-fel4 target
  sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
* [cmake](https://cmake.org/download/)
  
  CMake version `3.7.2` or greater is required.
  Binary releases are available from [cmake.org](https://cmake.org/download/).
  An example workflow for a recent binary installation on Ubuntu
  [can be found on StackExchange's askUbuntu](https://askubuntu.com/questions/355565/how-do-i-install-the-latest-version-of-cmake-from-the-command-line/865294#865294).
* [ninja-build](https://ninja-build.org/)
  
  Ninja version `1.7.1` or greater is required or greater is required due to the seL4 build system.
  Binary releases are available from [github](https://github.com/ninja-build/ninja/releases).
  
  Ubuntu users can typically install ninja using apt-get.
  
  ```bash
  sudo apt-get install ninja-build
  ```
* [Python Tooling](https://python.org/)
  
  The underlying seL4 build system also makes use of some Python tools.
  
  ```bash
  # Install python and pip, if you don't have them already
  sudo apt-get install python-pip
  
  pip install sel4-deps
  ```
* [xmlint](http://xmlsoft.org/xmllint.html)
  
  The underlying seL4 build system requires `xmlint`.
  
  ```bash
  sudo apt-get install libxml2-utils
  ```

### Building

This project is intended to be built in the context of the `cargo fel4` command, which manages
the piping of key environment variables relevant to the downstream project.
See the [cargo-fel4 repository](https://github.com/PolySync/cargo-fel4) for more direction, but the general idea is to create
a Rust project with `libsel4-sys` as a dependency, add a fel4.toml file, and to run `cargo fel4 build`.
While not recommended for general use, manual builds are possible. If environment variable `FEL4_ARTIFACT_PATH` is set, the kernel and simulation script
will be copied into the directory specified by the variable.

```bash
# Pull in seL4 related dependencies to the local filesystem
git submodule update --init

# Builds require that the `FEL4_MANIFEST_PATH` environment variable is set and
# includes a path pointing to a fel4.toml file, as specified by the `fel4-config` crate
# Additionally, the `RUST_TARGET_PATH` must be supplied, pointing to the directory that
# contains the Rust target specification JSON files relevant for the desired build target.
RUST_TARGET_PATH=$PWD/test_configs FEL4_MANIFEST_PATH=$PWD/test_configs/fel4.toml xargo rustc --target x86_64-sel4-fel4 -vv
```

### Installation

libsel4-sys may be included in your Rust project by adding the following
to your Cargo.toml dependencies.

```toml
libsel4-sys = { git = "https://github.com/PolySync/libsel4-sys.git", branch = "master" }
```

## Usage

The generated bindings should be treated as relatively ephemeral and dynamic compared
to most Rust libraries. The output is context-specific to the target (e.g. "armv7-sel4-fel4")
and the set of configuration flags derived from the input feL4 manifest file.

See the Rust docs for a surface-level overview of the raw APIs exposed.

```bash
RUST_TARGET_PATH=$PWD/test_configs FEL4_MANIFEST_PATH=$PWD/test_configs/fel4.toml xargo doc --target x86_64-sel4-fel4 -vv
```

### Examples

* Creating a seL4_CapRights_t instance
  ```rust
  extern crate libsel4_sys;
  use libsel4_sys::{seL4_Word, seL4_CapRights_new};
  let cap_rights = unsafe { seL4_CapRights_new(0 as seL4_Word, 1 as seL4_Word, 0 as seL4_Word); };
  ```

## Tests

Currently, all testing is done one level up, in the `cargo-fel4` repo,
which has the capability to auto-generate the appropriate test runner
code and exercise the resultant artifacts.

### Building

See the [cargo-fel4 repository](https://github.com/PolySync/cargo-fel4) for its
build and installation. 

### Running

Once `cargo-fel4` and the `libsel4-sys` dependencies are installed, you should be able to run:

```bash
cargo fel4 new tests && cd tests && cargo fel4 test build && cargo fel4 test simulate
```

# License

© 2018, PolySync Technologies, Inc.

* Jon Lamb [email](mailto:jlamb@polysync.io)
* Zack Pierce [email](mailto:zpierce@polysync.io)
* Dan Pittman [email](mailto:dpittman@polysync.io)

Please see the [LICENSE](./LICENSE) file for more details

