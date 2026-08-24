//
// Created by Krev3tka on 21.08.2026.
//

#include "vio_string.h"

VioString vio_str_from_literal(const char *s, size_t len) {
  return (VioString){.data = (char *)s, .len = len, .cap = 0};
}

VioString vio_str_concat(VioString a, VioString b) {
  size_t new_len = a.len + b.len;
  size_t total_bytes = offsetof(VioStringHeader, data) + new_len + 1;

  VioStringHeader *hdr = (VioStringHeader *)malloc(total_bytes);

  if (!hdr) {
    abort();
  }

  hdr->ref_count = 1;

  memcpy(hdr->data, a.data, a.len);
  memcpy(hdr->data + a.len, b.data, b.len);

  hdr->data[new_len] = '\0';

  return (VioString){.data = hdr->data, .len = new_len, .cap = new_len + 1};
}

void vio_str_retain(VioString s) {
  if (s.cap == 0 || !s.data)
    return;

  VioStringHeader *hdr =
      (VioStringHeader *)(s.data - offsetof(VioStringHeader, data));

  hdr->ref_count++;
}

void vio_str_release(VioString s) {
  if (s.cap == 0 || !s.data)
    return;

  VioStringHeader *hdr =
      (VioStringHeader *)(s.data - offsetof(VioStringHeader, data));

  hdr->ref_count--;

  if (hdr->ref_count == 0)
    free(hdr);
}
