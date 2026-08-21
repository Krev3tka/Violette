#include "vio_runtime/runtime.h"

int main(void) {
    VioString s1 = vio_str_from_literal("Hello, ", 7);
    VioString s2 = vio_str_from_literal("Violette", 8);

    VioString res = vio_str_concat(s1, s2);

    VioString input = vio_readln();

    vio_print_string(res);

    vio_print_string(input);

    vio_str_drop(res);
}
