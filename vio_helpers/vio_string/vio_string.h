//
// Created by Krev3tka on 21.08.2026.
//

#ifndef VIO_HELPERS_VIO_STRINGS_H
#define VIO_HELPERS_VIO_STRINGS_H

#include <stddef.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>



typedef struct {
  uint32_t ref_count;
  char data[];
} VioStringHeader;

typedef struct {
  const char *data;
  size_t len;
  size_t cap;
} VioString;

VioString vio_str_from_literal(const char *s, size_t len);
VioString vio_str_concat(VioString a, VioString b);
void vio_str_retain(VioString s);
void vio_str_release(VioString s);
int64_t vio_str_len(VioString s);
int64_t vio_str_get(VioString s, int64_t index);

#endif // VIO_HELPERS_VIO_STRINGS_H
