//
// Created by Krev3tka on 27.08.2026.
//

#include "vio_int.h"
#include <stdio.h>
#include <inttypes.h>
#include <stddef.h>

double vio_to_float(int64_t n) {
    return (double)n;
}

VioString vio_to_string(int64_t n) {
    int len = snprintf(NULL, 0, "%" PRId64, n);
    if (len < 0) {
        len = 0;
    }

    size_t total_bytes = offsetof(VioStringHeader, data) + (size_t)len + 1;
    VioStringHeader* hdr = malloc(total_bytes);

    if (!hdr) {
        abort();
    }

    hdr->ref_count = 1;

    snprintf(hdr->data, (size_t)len + 1, "%" PRId64, n);

    return (VioString){
        .data = hdr->data,
        .len = (size_t)len,
        .cap = (size_t)len + 1,
    };
}