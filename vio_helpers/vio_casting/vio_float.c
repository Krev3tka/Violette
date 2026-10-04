//
// Created by Krev3tka on 01.10.2026.
//

#include <stdint.h>
#include <stdio.h>
#include "../vio_string/vio_string.h"

int64_t vio_float_to_int(double f) {
    return (int64_t)f;
}

VioString vio_float_to_str(double f) {
    char *str = malloc(50);
    if (str == NULL) {
        return (VioString){ .data = NULL, .len = 0, .cap = 0 };
    }

    int64_t written = snprintf(str, 50, "%f", f);

    return (VioString){
        .data = str,
        .len = written,
        .cap = 50
    };
}
