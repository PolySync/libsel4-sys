# libsel4-sys

Builds the sel4 kernel and generates bindings around it,
as configured by a fel4 manifest.

## TODOs

- need to fix the cmakelist.txt file
- when to trigger reruns?

## Project Layout

```
libsel4-sys/
├── Cargo.toml              <-- CMake build configuration keys in toml tables
├── build.rs
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
TODO - update this README with constraints around environment variables

## Output Artifacts

If environment variable `FEL4_ARTIFACT_PATH` is set, the kernel and simulation script
will be copied into the directory specified by the variable.

## Down-stream Configuration

Down-stream consumers can specify a toml file that contains the CMake configuration
tables via the `FEL4_MANIFEST_PATH` variable. If not set, the default package configuration
will be used.


## Setup

Don't forget to run `git submodule update --init` before you start hacking.
