# CMake rules for glueing together the packaging
# of the seL4 kernel and C bindings

cmake_minimum_required(VERSION 3.7.2)

project(sel4_package NONE)

set(
    BuildWithCommonSimulationSettings
    OFF
    CACHE BOOL
    "Include only simulation compatible configurations")

set(rootservername sel4_package)

add_library(sel4_package ALIAS sel4)

# The current elfloader-tool CMakeList.txt doesn't set `-mgeneral-regs-only` for some reasone
# So we construct the base compile options for it here, `-ffreestanding` is required with it
if(KernelArchARM AND KernelSel4ArchAarch64 AND KernelPlatformTx1)
    set_property(TARGET elfloader PROPERTY COMPILE_OPTIONS "-ffreestanding;-mgeneral-regs-only")
endif()

ApplyData61ElfLoaderSettings()

if(BuildWithCommonSimulationSettings)
    ApplyCommonSimulationSettings()
    GenerateSimulateScript()
endif(BuildWithCommonSimulationSettings)

# Disable GC sections as it causes binaries to be stripped sometimes
set(UserLinkerGCSections OFF CACHE BOOL "" FORCE)

# This to mimick what DeclareRootserver() would do
set(
    IMAGE_NAME
    "${CMAKE_BINARY_DIR}/images/feL4img")

set(
    KERNEL_IMAGE_NAME
    "${CMAKE_BINARY_DIR}/images/kernel")

if("${KernelArch}" STREQUAL "x86")
    if(Kernel64)
        add_custom_command(
            OUTPUT "${KERNEL_IMAGE_NAME}"
            COMMAND ${CROSS_COMPILE_PREFIX}objcopy -O elf32-i386 $<TARGET_FILE:kernel.elf> "${KERNEL_IMAGE_NAME}"
            VERBATIM
            DEPENDS kernel.elf
            COMMENT "objcopy kernel into bootable elf"
        )
    else()
        add_custom_command(
            OUTPUT "${KERNEL_IMAGE_NAME}"
            COMMAND cp $<TARGET_FILE:kernel.elf> "${KERNEL_IMAGE_NAME}"
            VERBATIM
            DEPENDS kernel.elf
        )
    endif()

    add_custom_target(
        rootserver_image
        ALL DEPENDS
        "${KERNEL_IMAGE_NAME}"
        kernel.elf)
elseif("${KernelArch}" STREQUAL "arm")
    set(IMAGE_NAME "${CMAKE_BINARY_DIR}/images/kernel")
    if(NOT "${ElfloaderImage}" STREQUAL "elf")
        # If not an elf we construct an intermediate rule to do an objcopy to binary
        add_custom_command(OUTPUT "${IMAGE_NAME}"
            COMMAND ${CROSS_COMPILER_PREFIX}objcopy -O binary $<TARGET_FILE:elfloader> "${IMAGE_NAME}"
            DEPENDS $<TARGET_FILE:elfloader> elfloader
        )
    else ()
        add_custom_command(OUTPUT "${IMAGE_NAME}"
            COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:elfloader> "${IMAGE_NAME}"
            DEPENDS $<TARGET_FILE:elfloader> elfloader
        )
    endif()

    add_custom_target(
        rootserver_image
        ALL DEPENDS
        "${IMAGE_NAME}"
        elfloader
        ${rootservername})

    # WARNING - this will likely change as we migrate to a Rust project
    #
    # This is brittle, and won't complain if the `FEL4_ROOT_TASK_IMAGE_PATH` variable is not set.
    #
    # This is intentional and a step in the right direction until we have a
    # fully formed Rust bootstrapping tool.
    # It lets us get away with muli-stage building, where we can first
    # build kernel binary and libsel4.a without having a root-task image.
    # Then on further invocations (assuming environment variable is set) you can
    # have the elfloader-tool pick up the root-task image and link everthing together.
    set_property(TARGET rootserver_image PROPERTY ROOTSERVER_IMAGE "$ENV{FEL4_ROOT_TASK_IMAGE_PATH}")
endif()

# Get the basename of FEL4_ARTIFACT_PATH
get_filename_component(
    FEL4_ARTIFACT_BASENAME
    "$ENV{FEL4_ARTIFACT_PATH}"
    NAME)

# Mock the rootserver_image properties used by the simulate script building mechanism
set_property(TARGET rootserver_image PROPERTY IMAGE_NAME "${FEL4_ARTIFACT_BASENAME}/feL4img")
set_property(TARGET rootserver_image PROPERTY KERNEL_IMAGE_NAME "${FEL4_ARTIFACT_BASENAME}/kernel")

install(
    DIRECTORY
    $<TARGET_PROPERTY:sel4,INTERFACE_INCLUDE_DIRECTORIES>
    $<TARGET_PROPERTY:Configuration,INTERFACE_INCLUDE_DIRECTORIES>
    DESTINATION .)

install(
    FILES
    $<TARGET_FILE:sel4>
    DESTINATION lib)
