cmake_minimum_required(VERSION 3.7.2)

include(deps/seL4_tools/cmake-tool/base.cmake)

add_subdirectory(deps/musllibc musllibc)
add_subdirectory(deps/seL4_libs seL4_libs)
add_subdirectory(deps/util_libs util_libs)
add_subdirectory(package)

include(deps/seL4_tools/cmake-tool/configuration.cmake)
