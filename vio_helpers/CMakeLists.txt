cmake_minimum_required(VERSION 4.3)
project(vio_helpers C)

set(CMAKE_C_STANDARD 99)

add_executable(vio_helpers main.c
        vio_string/vio_string.h
        vio_string/vio_string.c
        vio_io/vio_print.c
        vio_io/vio_print.h
        vio_io/vio_readln.h
        vio_io/vio_println.c
        vio_io/vio_println.h
        vio_io/vio_readln.c
        vio_runtime/runtime.h
)
