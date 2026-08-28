# Violette Programming Language

> **Violette** — a statically typed compiled programming language combining the of Go/Kotlin/Swift with the speed of C/Rust.

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Backend](https://img.shields.io/badge/backend-C99%20%2F%20GNU99-blue.svg)](#)
[![Version](https://img.shields.io/badge/version-v0.4.0--alpha-purple.svg)](Violettech_v0.4.md)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

---

## Key Features

* Deterministic Memory Model: Fast reference counting (Perceus-style). *(not ready yet)*
* Null-Safety & Zero-Values: No hidden `null` references.
* UFCS Methods: Any function taking a struct or primitive as its first parameter behaves as a method (`p.distance()`, `"Violette".len()`).
* Sprout Pipelines (`~>`): Functional dataflow operator for conveyor-style transformations.
* Modular Standard Library: Embedded zero-cost prelude (`prelude.vio`)
* Modern Terminal Diagnostics: Card-based error reporting like in Gleam

---

## Code Examples

### 1. Methods & Pipeline Chaining
```violette
import math.{sqrt}

struct Point {
    x: float64,
    y: float64,
}

fun distance(p: Point, other: Point) [float64] {
    let dx = p.x - other.x
    let dy = p.y - other.y
    return sqrt((dx * dx) + (dy * dy))
}

fun main() {
    let p1 = Point { x: 0.0, y: 0.0 }
    let p2 = Point { x: 3.0, y: 4.0 }

    // Uniform Function Call Syntax:
    println(p1.distance(p2)) // 5.0

    // Multi-line fluent chaining:
    let result = (-64.0)
        .abs()
        .sqrt()
    
    println(result) // 8.0
}
```

### 2. Sprout Operator (`~>`)
```violette
fun fetch(url: string) [string] {
    return "payload"
}

fun parse(data: string) [string] {
    return "json"
}

fun validate(data: string) [bool] {
    return true
}

fun main() {
    // this string is the same as validate(parse(fetch("https://example.com")))
    let is_valid = "https://example.com" ~> fetch ~> parse ~> validate
    println(is_valid) // true
}
```

### 3. Strings & Prelude
```violette
fun main() {
    let text = "Violette"
    
    if !text.is_empty() {
        println("Length: " + text.len().to_string())
    }

    let clamped = clamp(150, 0, 100)
    println(clamped) // 100
}
```

---

## Roadmap

- [x] **Lexer & Tokenizer**
- [x] **Pratt Parser**
- [x] **Typechecker & Semantic Analysis** with immutability by default
- [x] **Flow Control Analysis**
- [x] **C-Transpiler Codegen** with FFI
- [ ] **Embedded Standard Library**
- [x] **Selective & Dotted Imports** (`import math.{abs, sqrt}`, `import math.hypot`)
- [ ] **Card-based Terminal Diagnostics** with ANSI styling and helpful hints
- [ ] **Tagged Unions** & Pattern Matching expressions (`match` + `Win/Fail`)
- [ ] **Error propagation** postfix operator (`|`)
- [ ] **Perceus In-Place Buffer Reuse (FBIP)** optimization
- [ ] **Native LLVM Backend** (Target v1.0)

---

## Building and Running

### Prerequisites
* Rust toolchain (Rust 1.80+)
* A standard C compiler (`clang` or `gcc`)

### Build from Source
```bash
git clone https://github.com/Krev3tka/Violette.git
cd Violette
cargo build --release
```

### Run an Example
```bash
# Run any .vio file directly:
./target/release/violette run examples/demo_showcase.vio
```

### Nix Environment (Reproducible)
```bash
nix develop
nix build
```

---

## Running Tests

Integration and snapshot tests are managed via the built-in test suite:

```bash
# Run Rust unit tests:
cargo test

# Run full integration test suite:
cd tools/Viotestte && go run main.go
```

---

## Support the Development

Violette is an independent open-source project. If you'd like to support the language design and compiler development:

* **ERC-20 / ETH:** `0x7ecc5C0a8A24dfCB885966a98aEc60fC8D736422`

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.