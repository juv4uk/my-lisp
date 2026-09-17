cmake_minimum_required(VERSION 3.21)
project(my_lisp_embed_static_consumer LANGUAGES C)

if(NOT WIN32)
  message(FATAL_ERROR "static consumer witness is Windows/MSVC-only")
endif()

set(MY_LISP_EMBED_STATIC "" CACHE FILEPATH "Cargo-produced my-lisp-embed static library")
if(NOT EXISTS "${MY_LISP_EMBED_STATIC}")
  message(FATAL_ERROR "MY_LISP_EMBED_STATIC must point at the Cargo-produced static library")
endif()

set(MY_LISP_NATIVE_LIBS "" CACHE STRING "Rust-reported native libraries required by the staticlib")
if(MY_LISP_NATIVE_LIBS STREQUAL "")
  message(FATAL_ERROR "MY_LISP_NATIVE_LIBS must come from rustc --print native-static-libs")
endif()

add_executable(my-lisp-embed-static-consumer main.c)
target_compile_options(my-lisp-embed-static-consumer PRIVATE /utf-8)
target_include_directories(
  my-lisp-embed-static-consumer
  PRIVATE
    "${CMAKE_CURRENT_LIST_DIR}/../../include"
)
target_link_libraries(
  my-lisp-embed-static-consumer
  PRIVATE
    "${MY_LISP_EMBED_STATIC}"
    ${MY_LISP_NATIVE_LIBS}
)
