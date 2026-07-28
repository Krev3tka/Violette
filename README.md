# Violette Programming Language

> **Violette** is a compiled language using Perceus reference counting and static typing.

[![Status](https://img.shields.io/badge/status-active_development-blue.svg)](#)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

---

## Features

* **Memory Management:** Violette uses RC with *Perceus Algorithm*
* **Null-Safety:** Violette doesn't have implicit `null`
* **Syntax:** Violette syntax is similar with Go with some features from Swift
* **Type System** Static, strict type system with algebraic types

---

## Code Examples

```violette
package main

fun main() {
    let message: string = "Hello, Violette!" // type annotations are not necessary
    print(message)
}
```

```violette
package main
fun apply(f: fun (int) [int], x: int) [int | string | bool] {
    return f(x)
}

fun main() {
    apply(print, 5)
}
```

```violette
package main

import (
    os,
    strings,
    time,
    math
)

fun now() [time.Time] {
    return time.Now()
}

// all instructions are permitted at top-level in main package, like in Swift

let content = os.ReadFile("log.txt")

let сurrentTime = now()
```

---

## Roadmap

  - [x] Language syntax and lexer

  - [x] AST parser (almost)

  - [x] Base typechecker

  - [ ] C-codegen for MVP phase (in progress)

  - [ ] Perceus Reference Counting IR Transformations

  - [ ] Codegen Pass

---

## Building from Source

To build the compiler locally, you need a working Rust toolchain:

```Bash
git clone [https://github.com/your-username/violette.git](https://github.com/your-username/violette.git)
cd violette
cargo build --release
```

---

## Support the Development

Violette is an open-source project. If you'd like to support the language design and compiler development:

  - **ERC:** 0x7ecc5C0a8A24dfCB885966a98aEc60fC8D736422

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
