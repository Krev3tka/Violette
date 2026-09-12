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
    printf("%s", s.data ? s.data : "");
}

void vio_print_bool(bool b) {
    printf("%s", b ? "true" : "false");
}

void vio_print_char(uint32_t c) {
    char buf[5] = {0};
    if (c <= 0x7F) {
        buf[0] = (char)c;
    } else if (c <= 0x7FF) {
        buf[0] = (char)(0xC0 | ((c >> 6) & 0x1F));
        buf[1] = (char)(0x80 | (c & 0x3F));
    } else if (c <= 0xFFFF) {
        buf[0] = (char)(0xE0 | ((c >> 12) & 0x0F));
        buf[1] = (char)(0x80 | ((c >> 6) & 0x3F));
        buf[2] = (char)(0x80 | (c & 0x3F));
    } else if (c <= 0x10FFFF) {
        buf[0] = (char)(0xF0 | ((c >> 18) & 0x07));
        buf[1] = (char)(0x80 | ((c >> 12) & 0x3F));
        buf[2] = (char)(0x80 | ((c >> 6) & 0x3F));
        buf[3] = (char)(0x80 | (c & 0x3F));
    }
    fputs(buf, stdout);
}