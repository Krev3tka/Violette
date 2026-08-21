//
// Created by Krev3tka on 21.08.2026.
//

#include "vio_string.h"

VioString vio_str_from_literal(const char* s, size_t len) {
    return (VioString){
        .data = (char*)s,
        .len = len,
        .cap = 0
    };
}

VioString vio_str_concat(VioString a, VioString b) {
    size_t new_len = a.len + b.len;
    char* buf = malloc(new_len + 1);

    if (!buf) {
        abort();
    }

    memcpy(buf, a.data, a.len);
    memcpy(buf + a.len, b.data, b.len);
    buf[new_len] = '\0';

    return (VioString) {
        .data = buf,
        .len = new_len,
        .cap = new_len + 1
    };
}

void vio_str_drop(VioString s) {
    if (s.cap > 0) free(s.data);
}