#include "vio_string.h"
#include <stdio.h>

int main(void) {
  VioString s1 = vio_str_from_literal("Hello, ", 7);
  VioString s2 = vio_str_from_literal("Violette!", 9);

  VioString res1 = vio_str_concat(s1, s2);
  printf("Concat: %s (len: %zu, cap: %zu)\n", res1.data, res1.len, res1.cap);

  VioString res2 = res1;
  vio_str_retain(res2);

  VioString s3 = vio_str_from_literal(" Runtime working!", 17);
  VioString res3 = vio_str_concat(res1, s3);
  printf("Result 2: %s\n", res3.data);

  vio_str_release(s1);
  vio_str_release(s2);
  vio_str_release(s3);

  vio_str_release(res1);
  vio_str_release(res2);
  vio_str_release(res3);

  printf("Test finished successfully!\n");
  return 0;
}
