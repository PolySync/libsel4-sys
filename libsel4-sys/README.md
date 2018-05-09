# seL4 Cargo Package

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

## Output Artifacts

If environment variable `HELIOS_ARTIFACT_PATH` is set, the kernel and simulation script
will be copied into the directory specified by the variable.

## Down-stream Configuration

Down-stream consumers can specify a toml file that contains the CMake configuration
tables via the `HELIOS_MANIFEST_PATH` variable. If not set, the default package configuration
will be used.

## Build Configuration

Example `fel4.toml`:

```
[fel4]
artifact-path = "images"
target-specs-path = "targets"
default-target = "x86_64-sel4-fel4"

[sel4-cmake-options]
KernelOptimisation = "-02"
KernelVerificationBuild = false
KernelBenchmarks = "none"
KernelDangerousCodeInjection = false
KernelFastpath = true
KernelMaxNumNodes = 1
KernelRetypeFanOutLimit = 256
KernelNumDomains = 1
KernelMaxNumBootinfoUntypedCaps = 230
KernelRootCNodeSizeBits = 19
LibSel4FunctionAttributes = "public"
KernelSupportPCID = false
BuildWithCommonSimulationSettings = true

[sel4-cmake-options.debug]
KernelDebugBuild = true
KernelPrinting = true

[sel4-cmake-options.release]
KernelDebugBuild = false
KernelPrinting = false

[sel4-cmake-options.x86_64-sel4-fel4]
KernelArch = "x86"
KernelX86Sel4Arch = "x86_64"

[sel4-cmake-options.arm-sel4-fel4]
CROSS_COMPILER_PREFIX = "arm-linux-gnueabihf-"
KernelArch = "arm"
KernelArmSel4Arch = "aarch32"
KernelARMPlatform = "sabre"

[sel4-cmake-options.aarch64-sel4-fel4]
CROSS_COMPILER_PREFIX = "aarch64-linux-gnu-"
KernelArch = "arm"
KernelArmSel4Arch = "aarch64"
KernelARMPlatform = "tx1"
```
