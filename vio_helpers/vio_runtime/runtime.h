//
// Created by Krev3tka on 21.08.2026.
//

#ifndef VIO_HELPERS_RUNTIME_H
#define VIO_HELPERS_RUNTIME_H

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <math.h>
#include <stddef.h>

/* ========== STRING ========== */

typedef struct {
  uint32_t ref_count;
  char data[];
} VioStringHeader;

typedef struct {
  const char *data;
  size_t len;
  size_t cap;
} VioString;

static inline VioString vio_str_from_literal(const char *s, size_t len) {
  return (VioString){.data = (char *)s, .len = len, .cap = 0};
}

static inline VioString vio_str_concat(VioString a, VioString b) {
  if (SIZE_MAX - a.len < b.len) {
    abort();
  }
  size_t new_len = a.len + b.len;
  size_t total_bytes = offsetof(VioStringHeader, data) + new_len + 1;

  VioStringHeader *hdr = (VioStringHeader *)malloc(total_bytes);

  if (!hdr) {
    abort();
  }

  hdr->ref_count = 1;

  if (a.len > 0 && a.data) {
    memcpy(hdr->data, a.data, a.len);
  }

  if (b.len > 0 && b.data) {
    memcpy(hdr->data + a.len, b.data, b.len);
  }

  hdr->data[new_len] = '\0';

  return (VioString){.data = hdr->data, .len = new_len, .cap = new_len + 1};
}

static inline int64_t vio_str_len(VioString s) {
    return s.len;
}

static inline void vio_str_retain(VioString s) {
  if (s.cap == 0 || !s.data)
    return;

  VioStringHeader *hdr =
      (VioStringHeader *)(s.data - offsetof(VioStringHeader, data));

  hdr->ref_count++;
}

static inline void vio_str_release(VioString s) {
  if (s.cap == 0 || !s.data)
    return;

  VioStringHeader *hdr =
      (VioStringHeader *)(s.data - offsetof(VioStringHeader, data));

  hdr->ref_count--;

  if (hdr->ref_count == 0)
    free(hdr);
}

static inline int64_t vio_str_get(VioString s, int64_t index) {
  if (index >= s.len || index < 0) {
    abort();
  }

  return (int64_t)s.data[index];
}

/* ========== IO ========== */

static inline void vio_print_int(int64_t x) {
    printf("%lld", (long long)x);
}

static inline void vio_print_float(double x) {
    printf("%g", x);
}

static inline void vio_print_string(VioString s) {
    printf("%.*s", (int)s.len, s.data ? s.data : "");
}

static inline void vio_print_bool(bool b) {
    printf("%s", b ? "true" : "false");
}

static inline void vio_print_char(uint32_t c) {
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

static inline void vio_println_int(int64_t x) {
    printf("%lld\n", (long long)x);
}

static inline void vio_println_float(double x) {
    printf("%g\n", x);
}

static inline void vio_println_string(VioString s) {
    printf("%s\n", s.data ? s.data : "");
}

static inline void vio_println_bool(bool b) {
    printf("%s\n", b ? "true" : "false");
}

static inline void vio_println_char(uint32_t c) {
    vio_print_char(c);
    fputc('\n', stdout);
}

static inline VioString vio_scanln() {
    char* line = NULL;
    size_t cap = 0;
    ssize_t nread = getline(&line, &cap, stdin);

    if (nread == -1) {
        free(line);
        return (VioString){
          .data = "",
          .len = 0,
          .cap = 0,
        };
    }

    if (nread > 0 && line[nread - 1] == '\n') {
        line[nread - 1] = '\0';
        nread--;
    }

    size_t total_bytes = offsetof(VioStringHeader, data) + (size_t)nread + 1;
    VioStringHeader* hdr = (VioStringHeader*)malloc(total_bytes);

    if (!hdr) {
        free(line);
        abort();
    }

    hdr->ref_count = 1;
    memcpy(hdr->data, line, (size_t)nread + 1);
    free(line);

    return (VioString){
        .data = hdr->data,
        .len = (size_t)nread,
        .cap = (size_t)nread + 1,
    };

}

/* ========== CASTING ========== */

static inline double vio_int_to_float(int64_t n) {
    return (double)n;
}

static inline VioString vio_int_to_str(int64_t n) {
    int len = snprintf(NULL, 0, "%" PRId64, n);
    if (len < 0) {
        len = 0;
    }

    size_t total_bytes = offsetof(VioStringHeader, data) + (size_t)len + 1;
    VioStringHeader* hdr = (VioStringHeader*)malloc(total_bytes);

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

static inline int64_t vio_str_to_int(VioString s) {
    return (int64_t)strtoll(s.data, NULL, 10);
}

static inline double vio_str_to_float(VioString s) {
    char *endptr;

    return strtod(s.data, &endptr);
}

static inline int64_t vio_float_to_int(double f) {
    return (int64_t)f;
}

static inline VioString vio_float_to_str(double f) {
    size_t buf_size = 50;
    VioStringHeader *str = (VioStringHeader*)malloc(offsetof(VioStringHeader, data) + buf_size);
    if (!str) {
        return (VioString){ .data = NULL, .len = 0, .cap = 0 };
    }
    str->ref_count = 1;
    int64_t written = snprintf(str->data, buf_size, "%g", f);

    return (VioString){
        .data = str->data,
        .len = (size_t)written,
        .cap = buf_size,
    };
}


#endif //VIO_HELPERS_RUNTIME_H
