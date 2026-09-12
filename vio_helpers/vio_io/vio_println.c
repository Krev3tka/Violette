//
// Created by Krev3tka on 21.08.2026.
//

#include "vio_println.h"
#include "vio_print.h"

void vio_println_int(int64_t x) {
    printf("%lld\n", (long long)x);
}

void vio_println_float(double x) {
    printf("%g\n", x);
}

void vio_println_string(VioString s) {
    printf("%s\n", s.data ? s.data : "");
}

void vio_println_bool(bool b) {
    printf("%s\n", b ? "true" : "false");
}

void vio_println_char(uint32_t c) {
    vio_print_char(c);
    fputc('\n', stdout);
}