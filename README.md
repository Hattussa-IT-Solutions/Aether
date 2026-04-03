# Aether Programming Language

A modern, expressive programming language with built-in parallelism, pattern matching, genetic evolution, GPU compute, and novel OOP concepts. Built in Rust.

## Quick Start

```bash
# Build from source
cargo build --release

# Run a program
cargo run -- run examples/hello.ae

# Start the REPL
cargo run -- repl

# Create a new project
cargo run -- new my-project
```

## Hello World

```
print("Hello from Aether!")
name = "World"
print("Hello, {name}!")
```

## Feature Highlights

### Pattern Matching
```
def classify(score) {
    match score {
        90..100 -> "A"
        80..90  -> "B"
        70..80  -> "C"
        _       -> "F"
    }
}
```

### Classes & Inheritance
```
class Animal {
    name: Str
    init(name) { self.name = name }
    def speak() { return "..." }
}

class Dog : Animal {
    def speak() { return "{self.name} says Woof!" }
}
```

### Parallel Blocks
```
parallel {
    users  = fetch_users()
    orders = fetch_orders()
    stats  = fetch_stats()
}
// All three run concurrently
```

### Pipeline Operator
```
result = data |> transform |> filter |> collect
```

### Functional Collections
```
nums = [1, 2, 3, 4, 5]
evens = nums.filter(x -> x % 2 == 0)
doubled = evens.map(x -> x * 2)
total = doubled.sum()
```

## Architecture

- **Lexer** — Full tokenizer with string interpolation, all operators, source locations
- **Parser** — Recursive descent with Pratt precedence for expressions
- **Type Checker** — Gradual typing with inference, #strict mode
- **Interpreter** — Tree-walk interpreter with all language features
- **Bytecode VM** — Stack-based VM for faster execution
- **GPU Codegen** — CUDA PTX, WebGPU WGSL, ROCm HIP generation
- **Package Manager** — `aether new`, `aether test`, `aether.toml`
- **VS Code Extension** — Syntax highlighting, snippets, bracket matching

## CLI Commands

| Command | Description |
|---------|-------------|
| `aether run <file.ae>` | Run a source file |
| `aether repl` | Interactive REPL |
| `aether new <name>` | Create new project |
| `aether test` | Run test files |
| `aether check <file.ae>` | Type-check a file |
| `aether --version` | Print version |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Apache License 2.0. See [LICENSE](LICENSE).
