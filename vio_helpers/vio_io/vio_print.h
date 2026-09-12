//
// Created by Krev3tka on 21.08.2026.
//

#ifndef VIO_HELPERS_VIO_PRINT_H
#define VIO_HELPERS_VIO_PRINT_H

#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdint.h>
#include "../vio_string/vio_string.h"

void vio_print_int(int64_t x);
void vio_print_float(double x);
void vio_print_string(VioString s);
void vio_print_bool(bool b);
void vio_print_char(uint32_t c);

#endif //VIO_HELPERS_VIO_PRINT_H
