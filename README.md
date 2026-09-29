# Violette Programming Language

> **Violette** — a statically typed compiled programming language combining the ergonomics of Kotlin/Swift, features from functional programming languages like Haskell/Idris and the speed of C/Rust.

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Backend](https://img.shields.io/badge/backend-C99%20%2F%20GNU99-blue.svg)](#)
[![Version](https://img.shields.io/badge/version-v0.2.0-purple.svg)](Violettech_v0.4.md)
[![License](https://img.shields.io/badge/license-BSD_3--Clause-blue.svg)](LICENSE)

---

## Key Features

* Deterministic Memory Model: Fast reference counting (Perceus-style). *(not ready yet)*
* Dependent types: No difference between expression and types (*not ready yet*)
* Null-Safety & Zero-Values: No hidden `null` references.
* Sprout Pipelines (`~>`): Functional dataflow operator for conveyor-style transformations.
* Modular Standard Library: Embedded zero-cost prelude (`prelude.vio`)
* Modern Terminal Diagnostics: Card-based error reporting like in Gleam

---

## Code Examples

### 1. Methods & Pipeline Chaining
```violette
using math (sqrt)

struct Point {
    x: Float64,
    y: Float64,
}

extend Point {
    func distance(self: Point, other: Point) -> Float64 {
        let dx = self.x - other.x
        let dy = self.y - other.y
        return sqrt((dx * dx) + (dy * dy))
    }
}

func main() {
    let p1 = Point { x: 0.0, y: 0.0 }
    let p2 = Point { x: 3.0, y: 4.0 }

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
func fetch(url: String) -> String {
    return "payload"
}

func parse(data: String) -> String {
    return "json"
}

func validate(data: String) -> Bool {
    return true
}

func main() {
    // this string is the same as validate(parse(fetch("https://example.com")))
    let is_valid = "https://example.com" ~> fetch ~> parse ~> validate
    println(is_valid) // true
}
```

### 3. Strings & Prelude
```violette
func main() {
    let text = "Violette"
    
    if !text.is_empty() {
        println("Length: " + text.len().to_string())
    }

    let clamped = clamp(150, 0, 100)
    println(clamped) // 100
}
```

### 4. Variant Types
```violette
variant Result {
    Win(Int),
    Fail(String),
}

let res = Result.Win(200)

let code = match res {
    Win(c) => c,
    Fail(msg) => 500,
}
```

### 5. Defer Statement
```violette
let file = open_file("log.txt")
defer file.close()
```

### 6. One-line If-Expression Syntactic Sugar
```violette
let sign = if x > 0: 1 else if x == 0: 0 else: -1
```

---

## Roadmap

- [x] **Lexer & Tokenizer**
- [x] **Pratt Parser**
- [x] **Typechecker & Semantic Analysis** with immutability by default
- [x] **Flow Control Analysis**
- [x] **C-Transpiler Codegen** with FFI
- [x] Block-scoped LIFO Defer
- [x] If-expressions with inline colon syntax
- [x] PascalCase unified type system
- [x] **Selective & Dotted Imports** (`using math (abs, sqrt)`, `using math (hypot)`)
- [x] **Tagged Unions** & Pattern Matching expressions (`match` + `Win/Fail`)
- [ ] **Embedded Standard Library**
- [ ] **Card-based Terminal Diagnostics** with helpful hints
- [ ] **Perceus In-Place Buffer Reuse (FBIP)** optimization

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

This project is licensed under the BSD 3-Clause License - see the [LICENSE](LICENSE) file for details.