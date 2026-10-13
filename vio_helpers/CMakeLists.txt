cmake_minimum_required(VERSION 4.3)
project(vio_helpers C)

set(CMAKE_C_STANDARD 99)

add_executable(vio_helpers main.c
        vio_string/vio_string.h
        vio_string/vio_string.c
        vio_io/vio_print.c
        vio_io/vio_print.h
        vio_io/vio_scanln.h
        vio_io/vio_println.c
        vio_io/vio_println.h
        vio_io/vio_scanln.c
        vio_runtime/runtime.h
        vio_casting/vio_int.h
        vio_casting/vio_int.c
        vio_casting/vio_string.h
        vio_casting/vio_string.c
        vio_casting/vio_float.h
        vio_casting/vio_float.c
        vio_array/vio_int_array.h
        vio_array/vio_int_array.c
)
