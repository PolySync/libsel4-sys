# rules for glueing together the packaging of the kernel and C bindings
# TODO - a proper rework is needed

cmake_minimum_required(VERSION 3.7.2)

#project(sel4_package C)
project(sel4_package NONE)

set(SIMULATION OFF CACHE BOOL "Include only simulation compatible configurations")

set(rootservername sel4_package)

add_library(sel4_package ALIAS sel4)

ApplyData61ElfLoaderSettings()

if(SIMULATION)
    ApplyCommonSimulationSettings()
    GenerateSimulateScript()
endif(SIMULATION)

# TODO - remove override
ApplyCommonReleaseVerificationSettings(OFF OFF)

# disable GC sections as it causes binaries to be stripped sometimes
set(UserLinkerGCSections OFF CACHE BOOL "" FORCE)

# doing this to mimick what DeclareRootserver() would do
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
endif()

#set_property(TARGET rootserver_image PROPERTY IMAGE_NAME "${IMAGE_NAME}")
#set_property(TARGET rootserver_image PROPERTY KERNEL_IMAGE_NAME "${KERNEL_IMAGE_NAME}")

# mock what we want to export for the build system, uses relative paths for now
set_property(TARGET rootserver_image PROPERTY IMAGE_NAME "images/feL4img")
set_property(TARGET rootserver_image PROPERTY KERNEL_IMAGE_NAME "images/kernel")

install(
    DIRECTORY
    $<TARGET_PROPERTY:sel4,INTERFACE_INCLUDE_DIRECTORIES>
    $<TARGET_PROPERTY:Configuration,INTERFACE_INCLUDE_DIRECTORIES>
    DESTINATION .)

install(
    FILES
    $<TARGET_FILE:sel4>
    DESTINATION lib)
