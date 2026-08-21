//
// Created by Krev3tka on 21.08.2026.
//

#include "vio_readln.h"

VioString vio_readln() {
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

    VioString vio_input = {
        .data = line,
        .len = (size_t)nread,
        .cap = cap
    };

    return vio_input;
}