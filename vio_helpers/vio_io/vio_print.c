//
// Created by Krev3tka on 21.08.2026.
//

#include "vio_print.h"

void vio_print_int(int64_t x) {
    printf("%lld", (long long)x);
}

void vio_print_float(double x) {
    printf("%g", x);
}

void vio_print_string(VioString s) {
    printf("%s", s.data);
}

void vio_print_bool(bool b) {
    printf("%s", b ? "true" : "false");
}