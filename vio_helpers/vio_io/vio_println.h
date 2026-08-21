//
// Created by Krev3tka on 21.08.2026.
//

#ifndef VIO_HELPERS_VIO_PRINTLN_H
#define VIO_HELPERS_VIO_PRINTLN_H

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include "../vio_string/vio_string.h"

void vio_println_int(int64_t x);
void vio_println_float(double x);
void vio_println_string(VioString s);
void vio_println_bool(bool b);

#endif //VIO_HELPERS_VIO_PRINTLN_H
