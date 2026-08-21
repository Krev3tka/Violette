//
// Created by Krev3tka on 21.08.2026.
//


#ifndef VIO_HELPERS_VIO_STRINGS_H
#define VIO_HELPERS_VIO_STRINGS_H

#include <stdlib.h>
#include <string.h>

typedef struct {
    char* data;
    size_t len;
    size_t cap;
} VioString;

VioString vio_str_from_literal(const char* s, size_t len);
VioString vio_str_concat(VioString a, VioString b);
void vio_str_drop(VioString s);

#endif //VIO_HELPERS_VIO_STRINGS_H
