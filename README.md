<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0--alpha-blueviolet?style=for-the-badge" alt="Version">
  <img src="https://img.shields.io/badge/language-Rust-orange?style=for-the-badge&logo=rust" alt="Built with Rust">
  <img src="https://img.shields.io/badge/tests-270%20passing-brightgreen?style=for-the-badge" alt="Tests">
  <img src="https://img.shields.io/badge/license-Apache%202.0-blue?style=for-the-badge" alt="License">
</p>

<h1 align="center">Aether Programming Language</h1>

<p align="center">
  <strong>A modern, expressive programming language with built-in parallelism, pattern matching, genetic evolution, GPU compute, and 15 novel OOP concepts.</strong>
</p>

<p align="center">
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-why-aether">Why Aether</a> •
  <a href="#-language-tour">Language Tour</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-contributing">Contributing</a>
</p>

---

## Why Aether?

Modern software demands **concurrency**, **safety**, and **expressiveness** — but most languages force you to choose between them. Aether eliminates that trade-off.

| Problem | Aether's Solution |
|---|---|
| Concurrency is hard | `parallel {}` blocks — tasks run concurrently by default |
| Boilerplate-heavy OOP | Computed properties, observed fields, operator overloading — all built in |
| No built-in optimization | `genetic class` with `evolve {}` — evolutionary optimization is a language primitive |
| GPU programming is separate | `@gpu` decorator compiles functions to CUDA/WebGPU/ROCm automatically |
| Pattern matching is an afterthought | First-class `match` with destructuring, ranges, guards, and exhaustiveness |
| Error handling is messy | `Result<T,E>` with `?` propagation, `try/catch`, and `??` nil coalescing |

Aether is **Python-readable**, **Rust-safe**, and **Go-concurrent** — in one language.

---

## Quick Start

```bash
# One-line install (Linux/macOS)
curl -sSf https://raw.githubusercontent.com/Hattussa-IT-Solutions/Aether/main/install.sh | sh

# Or build from source
git clone https://github.com/Hattussa-IT-Solutions/Aether.git
cd Aether
cargo build --release

# Run your first program
echo 'print("Hello from Aether!")' > hello.ae
aether run hello.ae

# Start the interactive REPL
aether repl
```

---

## Language Tour

### Variables & Types

```ruby
x = 42                    # Mutable, type inferred
let y = 10                # Immutable
const MAX = 100           # Compile-time constant
name: Str = "Aether"     # Explicit type annotation

#strict                   # Requires types on everything
let z: Int = x + y
```

### Functions & Lambdas

```ruby
def add(a: Int, b: Int) -> Int {
    return a + b
}

def double(x) = x * 2          # Expression body

greet = name -> "Hello, {name}!"   # Lambda

# Higher-order functions
[1, 2, 3, 4, 5]
    .filter(x -> x > 2)
    .map(x -> x ** 2)
    .sum()                      # => 50
```

### Pattern Matching

```ruby
def classify(score) {
    match score {
        90..100  -> "A"
        80..90   -> "B"
        70..80   -> "C"
        _        -> "F"
    }
}

match result {
    Ok(data)           -> process(data)
    Err(msg) if retry  -> fetch_again()
    Err(msg)           -> log(msg)
}

match shape {
    .Circle(r)    -> PI * r ** 2
    .Rect(w, h)   -> w * h
}
```

### Classes & Inheritance

```ruby
class Animal {
    name: Str
    age: Int = 0

    init(name) { self.name = name }

    def speak() { return "..." }
    def birthday() { self.age += 1 }

    # Computed property — recalculates on every access
    description: Str => "{self.name}, age {self.age}"
}

class Dog : Animal {
    tricks: Int = 0

    def speak() { return "{self.name} says Woof!" }
    def learn() { self.tricks += 1 }

    # Operator overloading
    operator ==(other: Dog) -> Bool {
        return self.name == other.name
    }
}

rex = Dog("Rex")
rex.learn()
rex.birthday()
print(rex.description)     # "Rex, age 1"
print(rex.speak())         # "Rex says Woof!"
```

### Parallel Execution

```ruby
# All tasks run concurrently — not sequentially
parallel {
    users  = fetch_users()
    orders = fetch_orders()
    stats  = compute_stats()
}
# Takes ~1x time, not 3x

# Race — first result wins
result = parallel.race {
    fast = cache_lookup(id)
    slow = db_query(id)
}
```

### Error Handling

```ruby
# Result type with ? propagation
def load_config(path: Str) -> Result<Config, Error> {
    data = fs_read(path)?           # Returns Err early if file missing
    config = json_decode(data)?     # Returns Err early if invalid JSON
    return Ok(config)
}

# Try/catch for imperative style
try {
    risky_operation()
} catch IOError as e {
    log("IO failed: {e}")
} finally {
    cleanup()
}

# Nil safety
city = user?.address?.city ?? "Unknown"
```

### Genetic Evolution

```ruby
genetic class Strategy {
    chromosome params {
        gene threshold: Float = 0.5 { range 0.0..1.0 }
        gene window: Int = 20 { range 5..200 }
    }

    fitness(data: Float) -> Float {
        correct = 0
        for val in data {
            if val > self.threshold { correct += 1 }
        }
        return correct
    }
}

# Evolve finds optimal parameters automatically
best = evolve Strategy {
    population: 50
    generations: 100
    mutation_rate: 0.1
    fitness on data: training_data
}

print("Best threshold: {best.threshold}")   # Optimized value
print("Fitness: {best.last_fitness}")        # 10/10
```

### Novel OOP Concepts

```ruby
# Reactive properties — auto-recompute when dependencies change
class Cart {
    items: Product[] = []
    reactive total: Float = items.sum(p -> p.price)
}

# Temporal properties — automatic history tracking
class Sensor {
    temporal(keep: 100) temperature: Float = 20.0
}
sensor.temperature = 25.0
sensor.temperature = 22.0
print(sensor.previous("temperature"))   # 25.0
print(sensor.history("temperature"))    # [20.0, 25.0]

# Mutation tracking with undo/redo
class Document {
    mutation(tracked, undoable: 50) content: Str = ""
}
doc = Document()
doc.content = "Hello"
doc.content = "Hello World"
doc.rollback("content")                 # Back to "Hello"

# Weave — inject behavior into all methods
weave Logged {
    before { log("method called") }
    after  { log("method done") }
}
class API with Logged {
    def get_users() { return db.query("users") }
    # Logging is automatically injected!
}

# Extend any type
extend Int {
    def is_even() -> Bool = self % 2 == 0
}
42.is_even()    # true
```

### Pipeline Operator

```ruby
result = raw_data
    |> parse
    |> validate
    |> transform
    |> (x -> x.filter(valid))
    |> save
```

### Collections & Comprehensions

```ruby
# List comprehension
squares = [x ** 2 for x in 0..10]
evens = [x for x in data if x % 2 == 0]

# Rich collection methods
nums = [5, 3, 1, 4, 2]
nums.sort()           # [1, 2, 3, 4, 5]
nums.reduce(0, (a, b) -> a + b)   # 15
nums.zip(["a", "b", "c"])         # [(5,"a"), (3,"b"), (1,"c")]
nums.chunks(2)                     # [[5,3], [1,4], [2]]

# Map, Set, Tuple
config = {"host": "localhost", "port": 8080}
unique = {1, 2, 3}.union({3, 4, 5})    # {1, 2, 3, 4, 5}
point = (3.14, 2.71)
```

---

## Installation

### One-Line Install (Recommended)

**Linux / macOS:**
```bash
curl -sSf https://raw.githubusercontent.com/Hattussa-IT-Solutions/Aether/main/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Hattussa-IT-Solutions/Aether/main/install.ps1 | iex
```

### Build from Source

```bash
# Requires Rust 1.70+
git clone https://github.com/Hattussa-IT-Solutions/Aether.git
cd Aether
cargo build --release
# Binary at: target/release/aether
```

### Docker

```bash
docker build -t aether .
docker run --rm -v $(pwd):/work aether run /work/hello.ae
```

### Other Methods

| Method | Command |
|--------|---------|
| Homebrew | `brew install aether` *(coming soon)* |
| Snap | `snap install aether` *(coming soon)* |
| Debian | `sudo dpkg -i aether_0.1.0_amd64.deb` |
| npm/npx | `npx aether-lang run hello.ae` |

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `aether run <file.ae>` | Execute a source file |
| `aether run <file.ae> --vm` | Execute using the bytecode VM |
| `aether jit <file.ae>` | Compile and run via Cranelift JIT |
| `aether repl` | Interactive REPL with history |
| `aether new <name>` | Create a new project |
| `aether test [dir]` | Run test files |
| `aether check <file.ae>` | Type-check (use `#strict` for full checking) |
| `aether --version` | Print version |

---

## Architecture

```
Source (.ae)
    │
    ▼
┌─────────┐    ┌──────────┐    ┌──────────────┐
│  Lexer   │───▶│  Parser  │───▶│  AST Nodes   │
└─────────┘    └──────────┘    └──────┬───────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                  ▼
             ┌────────────┐   ┌────────────┐    ┌─────────────┐
             │ Interpreter│   │ Bytecode   │    │  Cranelift   │
             │ (tree-walk)│   │ VM         │    │  JIT/AOT     │
             └────────────┘   └────────────┘    └─────────────┘
                                                        │
                                      ┌─────────────────┼──────────┐
                                      ▼                 ▼          ▼
                               ┌──────────┐    ┌──────────┐ ┌──────────┐
                               │ CUDA PTX │    │  WebGPU  │ │ ROCm HIP │
                               └──────────┘    │  WGSL    │ └──────────┘
                                               └──────────┘
```

| Component | Lines | Description |
|-----------|-------|-------------|
| Lexer | ~500 | Tokenizes all Aether syntax including string interpolation |
| Parser | ~2,500 | Recursive descent with Pratt precedence for expressions |
| AST | ~900 | Complete node types for the full language |
| Interpreter | ~3,500 | Tree-walk execution with all language features |
| Type Checker | ~200 | Gradual typing with `#strict` mode |
| Bytecode VM | ~500 | Stack-based VM (runs `hello.ae`) |
| Cranelift JIT | ~100 | Native compilation for arithmetic programs |
| GPU Codegen | ~300 | CUDA PTX, WebGPU WGSL, ROCm HIP from AST |
| Standard Library | ~800 | String, List, Map, Set, Math, IO, JSON, Time, Tensor |
| Package Manager | ~200 | `aether new`, `aether test`, `aether.toml` |
| REPL | ~100 | Interactive with multiline and history |

**Total: ~11,000 lines of Rust | 270 tests | 8 example programs**

---

## Standard Library

### Built-in Types & Methods

<details>
<summary><strong>Str</strong> — 15 methods</summary>

`len()`, `upper()`, `lower()`, `trim()`, `split(sep)`, `contains(sub)`, `starts_with(s)`, `ends_with(s)`, `replace(old, new)`, `slice(start, end)`, `chars()`, `repeat(n)`, `parse_int()`, `parse_float()`, `join(list)`

</details>

<details>
<summary><strong>List</strong> — 25+ methods</summary>

`len()`, `push(item)`, `pop()`, `first()`, `last()`, `map(fn)`, `filter(fn)`, `reduce(init, fn)`, `sort()`, `reverse()`, `contains(item)`, `index_of(item)`, `sum()`, `min()`, `max()`, `any(fn)`, `all(fn)`, `unique()`, `flat_map(fn)`, `zip(other)`, `chunks(n)`, `insert(i, item)`, `remove(i)`, `join(sep)`

</details>

<details>
<summary><strong>Map</strong> — 8 methods</summary>

`len()`, `get(key)`, `set(key, val)`, `remove(key)`, `contains_key(key)`, `keys()`, `values()`, `entries()`

</details>

<details>
<summary><strong>Set</strong> — 7 methods</summary>

`len()`, `contains(item)`, `insert(item)`, `remove(item)`, `union(other)`, `intersect(other)`, `difference(other)`

</details>

<details>
<summary><strong>Math</strong> — 20+ functions</summary>

`sqrt`, `abs`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `pow`, `log`, `log2`, `log10`, `exp`, `floor`, `ceil`, `round`, `trunc`, `clamp`, `cbrt`, `min`, `max`, `random`

Constants: `PI`, `E`, `TAU`, `INF`

</details>

<details>
<summary><strong>Tensor</strong> — 12 operations</summary>

`Tensor_zeros(shape)`, `Tensor_ones(shape)`, `Tensor_random(shape)`, `Tensor_from_list(data)`, `tensor_add(a, b)`, `tensor_mul(a, b)`, `tensor_matmul(a, b)`, `tensor_transpose(t)`, `tensor_reshape(t, shape)`, `tensor_sum(t)`, `tensor_mean(t)`, `tensor_shape(t)`

</details>

### Additional Modules

| Module | Functions |
|--------|-----------|
| **File I/O** | `fs_read`, `fs_write`, `fs_exists`, `fs_lines` |
| **Time** | `time_now`, `time_sleep`, `time_millis` |
| **JSON** | `json_encode`, `json_decode` |

---

## Examples

The `examples/` directory contains complete, runnable programs:

| File | What it demonstrates |
|------|---------------------|
| [`hello.ae`](examples/hello.ae) | Hello world, string interpolation |
| [`basics.ae`](examples/basics.ae) | Variables, math, functions, loops, classes, lambdas, error handling |
| [`control_flow.ae`](examples/control_flow.ae) | If/else, match, for/loop variants, break/next, guard |
| [`error_handling.ae`](examples/error_handling.ae) | Result types, `?` propagation, try/catch/finally |
| [`pattern_matching.ae`](examples/pattern_matching.ae) | Literal, range, destructure, guard, enum variant patterns |
| [`classes.ae`](examples/classes.ae) | OOP, inheritance, interfaces, structs, enums, collections |
| [`genetic_evolution.ae`](examples/genetic_evolution.ae) | Genetic classes, evolve block, crossover, fitness optimization |
| [`full_stack.ae`](examples/full_stack.ae) | Every major feature in one program |

Run any example:
```bash
aether run examples/full_stack.ae
```

---

## VS Code Extension

Full IDE support in a separate repo: **[aether-vscode](https://github.com/Hattussa-IT-Solutions/aether-vscode)**

Features:
- Full syntax highlighting (keywords, operators, types, decorators, string interpolation)
- 10 code snippets (class, def, for, match, parallel, try/catch, etc.)
- Auto-closing brackets and comment toggling

---

## Roadmap

- [x] Core language (lexer, parser, interpreter)
- [x] Pattern matching with destructuring
- [x] Classes, structs, enums, interfaces
- [x] Parallel execution blocks
- [x] Genetic evolution engine
- [x] 15 novel OOP concepts
- [x] Bytecode VM
- [x] Cranelift JIT compilation
- [x] GPU codegen (CUDA PTX, WebGPU WGSL, ROCm HIP)
- [x] Tensor library
- [x] REPL, package manager, type checker
- [x] VS Code extension
- [ ] Full async/await with Tokio runtime
- [ ] Python interop via pyo3
- [ ] Language Server Protocol (LSP)
- [ ] WebAssembly compilation target
- [ ] Comprehensive documentation site

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

```bash
# Clone and build
git clone https://github.com/Hattussa-IT-Solutions/Aether.git
cd Aether
cargo build

# Run tests
cargo test

# Run an example
cargo run -- run examples/hello.ae
```

---

## License

Apache License 2.0 — see [LICENSE](LICENSE).

---

<p align="center">
  Built with Rust by <a href="https://github.com/Hattussa-IT-Solutions">Hattussa IT Solutions</a>
</p>
