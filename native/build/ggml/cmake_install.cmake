# Install script for directory: /home/neon/Documents/CODE LOCAL/forge/native/src/ggml

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "/usr/local")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "Release")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

# Install shared libraries without execute permission?
if(NOT DEFINED CMAKE_INSTALL_SO_NO_EXE)
  set(CMAKE_INSTALL_SO_NO_EXE "0")
endif()

# Is this installation the result of a crosscompile?
if(NOT DEFINED CMAKE_CROSSCOMPILING)
  set(CMAKE_CROSSCOMPILING "FALSE")
endif()

# Set path to fallback-tool for dependency-resolution.
if(NOT DEFINED CMAKE_OBJDUMP)
  set(CMAKE_OBJDUMP "/usr/bin/objdump")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("/home/neon/Documents/CODE LOCAL/forge/native/build/ggml/src/cmake_install.cmake")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  foreach(file
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib64/libggml.so.0.25.1"
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib64/libggml.so.0"
      )
    if(EXISTS "${file}" AND
       NOT IS_SYMLINK "${file}")
      file(RPATH_CHECK
           FILE "${file}"
           RPATH "")
    endif()
  endforeach()
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib64" TYPE SHARED_LIBRARY FILES
    "/home/neon/Documents/CODE LOCAL/forge/native/build/bin/libggml.so.0.25.1"
    "/home/neon/Documents/CODE LOCAL/forge/native/build/bin/libggml.so.0"
    )
  foreach(file
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib64/libggml.so.0.25.1"
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib64/libggml.so.0"
      )
    if(EXISTS "${file}" AND
       NOT IS_SYMLINK "${file}")
      file(RPATH_CHANGE
           FILE "${file}"
           OLD_RPATH "/home/neon/Documents/CODE LOCAL/forge/native/build/bin:"
           NEW_RPATH "")
      if(CMAKE_INSTALL_DO_STRIP)
        execute_process(COMMAND "/usr/bin/strip" "${file}")
      endif()
    endif()
  endforeach()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib64" TYPE SHARED_LIBRARY FILES "/home/neon/Documents/CODE LOCAL/forge/native/build/bin/libggml.so")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include" TYPE FILE FILES
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-cpu.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-alloc.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-backend.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-blas.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-cann.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-cpp.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-cuda.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-opt.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-metal.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-rpc.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-virtgpu.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-sycl.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-vulkan.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-webgpu.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-zendnn.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/ggml-openvino.h"
    "/home/neon/Documents/CODE LOCAL/forge/native/src/ggml/include/gguf.h"
    )
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  foreach(file
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib64/libggml-base.so.0.25.1"
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib64/libggml-base.so.0"
      )
    if(EXISTS "${file}" AND
       NOT IS_SYMLINK "${file}")
      file(RPATH_CHECK
           FILE "${file}"
           RPATH "")
    endif()
  endforeach()
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib64" TYPE SHARED_LIBRARY FILES
    "/home/neon/Documents/CODE LOCAL/forge/native/build/bin/libggml-base.so.0.25.1"
    "/home/neon/Documents/CODE LOCAL/forge/native/build/bin/libggml-base.so.0"
    )
  foreach(file
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib64/libggml-base.so.0.25.1"
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib64/libggml-base.so.0"
      )
    if(EXISTS "${file}" AND
       NOT IS_SYMLINK "${file}")
      if(CMAKE_INSTALL_DO_STRIP)
        execute_process(COMMAND "/usr/bin/strip" "${file}")
      endif()
    endif()
  endforeach()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib64" TYPE SHARED_LIBRARY FILES "/home/neon/Documents/CODE LOCAL/forge/native/build/bin/libggml-base.so")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib64/cmake/ggml" TYPE FILE FILES
    "/home/neon/Documents/CODE LOCAL/forge/native/build/ggml/ggml-config.cmake"
    "/home/neon/Documents/CODE LOCAL/forge/native/build/ggml/ggml-config-version.cmake"
    )
endif()

string(REPLACE ";" "\n" CMAKE_INSTALL_MANIFEST_CONTENT
       "${CMAKE_INSTALL_MANIFEST_FILES}")
if(CMAKE_INSTALL_LOCAL_ONLY)
  file(WRITE "/home/neon/Documents/CODE LOCAL/forge/native/build/ggml/install_local_manifest.txt"
     "${CMAKE_INSTALL_MANIFEST_CONTENT}")
endif()
