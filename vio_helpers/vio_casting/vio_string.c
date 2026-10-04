//
// Created by Krev3tka on 01.10.2026.
//

#include "../vio_casting/vio_string.h"
#include <stdio.h>
#include <inttypes.h>
#include <stddef.h>
#include <stdlib.h>
#include <stdint.h>

int64_t vio_str_to_int(VioString s) {
    int64_t res = 0;

    for (int i = 0; i < s.len; i++) {
        res = res * 10 + (s.data[i] - '0');
    }

    return res;
}

double vio_str_to_float(VioString s) {
    char *endptr;

    return strtof(s.data, &endptr);
}