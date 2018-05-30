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

* [rust](https://github.com/rust-lang-nursery/rustup.rs) (nightly)
* [xargo](https://github.com/japaric/xargo) (for cross-compiling)
* [gcc/g++ cross compilers](https://gcc.gnu.org/) (for ARM targets)
* [cmake](https://cmake.org/download/) (for seL4's build)
* [ninja-build](https://ninja-build.org/) (for seL4's build)
* [python](https://python.org/) (for seL4's build)
* [xmlint](http://xmlsoft.org/xmllint.html) (for seL4's build)

### Building

This project is intended to be built in the context of the `cargo fel4` command, which manages
the piping of key environment variables relevant to the downstream project.

* Install [Rust Nightly](https://github.com/rust-lang-nursery/rustup.rs)
  ```bash
  # Download the rustup install script
  wget https://static.rust-lang.org/rustup/rustup-init.sh
  
  # Install rustup
  chmod +x rustup-init.sh
  sh rustup-init.sh
  
  rustup install nightly
  ```
* Install [xargo](https://github.com/japaric/xargo)
  ```bash
  # Xargo requires rust-src component
  rustup component add --toolchain nightly rust-src
  
  # Install Xargo
  cargo +nightly install xargo
  ```
* Install the [gnu cross compilers](https://gcc.gnu.org/)
  ```bash
  # Used by the armv7-sel4-fel4 target
  sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
  
  # Used by the aarch64-sel4-fel4 target
  sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
  ```
* Install Python, pip, and a sel4-specific pip package.
  ```bash
  # Install python and pip, if you don't have them already
  sudo apt-get install python-pip
  
  pip install sel4-deps
  ```
* Install [cmake](https://cmake.org/download/)
  
  CMake version `3.7.2` or greater is required.
  Binary releases are available from [cmake.org](https://cmake.org/download/).
  
  An example workflow for a recent binary installation on Ubuntu
  [can be found on StackExchange's askUbuntu](https://askubuntu.com/questions/355565/how-do-i-install-the-latest-version-of-cmake-from-the-command-line/865294#865294).
  
  Alternately, you can use Python's `pip` tool to install the latest cmake.
  ```bash
  sudo pip install --upgrade cmake
  ```
* Install [ninja-build](https://ninja-build.org/)
  Ninja version `1.7.1` or greater is required or greater is required due to the seL4 build system.
  Binary releases are available from [github](https://github.com/ninja-build/ninja/releases).
  
  Ubuntu users can typically install ninja using apt-get.
  
  ```bash
  sudo apt-get install ninja-build
  ```
* [xmlint](http://xmlsoft.org/xmllint.html)
  
  The underlying seL4 build system requires `xmlint`.
  
  ```bash
  sudo apt-get install libxml2-utils
  ```
* Install [cargo-fel4](https://github.com/PolySync/cargo-fel4) using the directions from that repository.
* Use `cargo-fel4` to create a new feL4 project, which will automatically include `libsel4-sys` as a dependency to build
  ```bash
  cargo fel4 new demo_project
  cd demo_project
  cargo fel4 build
  ```
* Manual builds are available as an alternative to using `cargo-fel4`, though are not recommended for general use.
* Clone the libsel4-sys repository
  ```bash
  git clone git@github.com:PolySync/libsel4-sys.git
  cd libsel4-sys
  ```
* Pull in seL4 related dependencies to the local filesystem
  ```bash
  git submodule update --init
  ```
* Manual builds require that the `FEL4_MANIFEST_PATH` environment variable is set and
  includes a path pointing to a fel4.toml file, as specified by the `fel4-config` crate.
  Additionally, the `RUST_TARGET_PATH` must be supplied, pointing to the directory that
  contains the Rust target specification JSON files relevant for the desired build target.
  ```bash
  RUST_TARGET_PATH=$PWD/test_configs FEL4_MANIFEST_PATH=$PWD/test_configs/fel4.toml xargo rustc --target x86_64-sel4-fel4 -vv
  ```

### Installation

libsel4-sys may be included in your Rust project by including it in your Cargo.toml.

* In the relevant `[dependencies]` section:
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
cargo fel4 new tests
cd tests
cargo fel4 test build
cargo fel4 test simulate
```

# License

© 2018, PolySync Technologies, Inc.

* Jon Lamb [email](mailto:jlamb@polysync.io)
* Zack Pierce [email](mailto:zpierce@polysync.io)
* Dan Pittman [email](mailto:dpittman@polysync.io)

Please see the [LICENSE](./LICENSE) file for more details

