<p align="center">
  <img src="https://img.shields.io/badge/version-0.5.0-blueviolet?style=for-the-badge" alt="Version">
  <img src="https://img.shields.io/badge/language-Rust-orange?style=for-the-badge&logo=rust" alt="Built with Rust">
  <img src="https://img.shields.io/badge/tests-663%20passing-brightgreen?style=for-the-badge" alt="Tests">
  <img src="https://img.shields.io/badge/stdlib-133%20methods-blue?style=for-the-badge" alt="Stdlib">
  <img src="https://img.shields.io/badge/license-Apache%202.0-blue?style=for-the-badge" alt="License">
</p>

<h1 align="center">Aether Programming Language</h1>

<p align="center">
  <strong>A modern programming language with built-in parallelism, 37 stdlib modules, genetic evolution, GPU compute, Python interop, and features no other language has — TTL variables, reactive properties, and temporal state tracking.</strong>
</p>

<p align="center">
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-why-aether">Why Aether</a> •
  <a href="#-language-tour">Language Tour</a> •
  <a href="#-built-in-standard-library">Standard Library</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-contributing">Contributing</a>
</p>

---

## Why Aether?

| Problem | How other languages solve it | How Aether solves it |
|---------|-----|------|
| Need HTTP requests | `pip install requests` | Built in: `http_get(url)` |
| Need colored terminal output | `pip install colorama` | Built in: `term_color("text", "green")` |
| Need CSV parsing | `import csv` + boilerplate | Built in: `data_from_csv(text)` |
| Need SHA256 hashing | `pip install hashlib` | Built in: `crypto_sha256("data")` |
| Need regex | `import re` | Built in: `regex_find_all(pattern, text)` |
| Need JSON | `import json` | Built in: `json_encode(data)` |
| Need UUID generation | `pip install uuid` | Built in: `uuid_v4()` |
| Need base64 encoding | `import base64` | Built in: `base64_encode("data")` |
| Need concurrency | `import threading` + locks | Built in: `parallel { }` blocks |
| Need data analysis | `pip install pandas numpy` | Built in: `data_from_csv()`, `stats_mean()`, `viz_bar()` |
| Need caching with expiry | Build it yourself | Built in: TTL variables — `ttl_set("key", val, 30.0)` |
| Need to optimize parameters | `pip install scipy` | Built in: `evolve Strategy { population: 100 }` |

**Zero installs. Zero pip. Zero npm. Just `use`.**

---

## Quick Start

```bash
# One-line install (Linux/macOS)
curl -sSf https://raw.githubusercontent.com/Hattussa-IT-Solutions/Aether/main/install.sh | sh

# Or build from source (requires Rust 1.70+)
git clone https://github.com/Hattussa-IT-Solutions/Aether.git
cd Aether && cargo build --release

# Run a program
aether run examples/hello.ae

# Start the REPL
aether repl
```

---

## Language Tour

### Variables & Strings
```ruby
name = "Aether"
version = 0.5
print("Welcome to {name} v{version}!")   # String interpolation built in

let immutable = 42      # Immutable
const MAX = 1000        # Compile-time constant
```

### Functions & Lambdas
```ruby
def fibonacci(n) {
    if n <= 1 { return n }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

double = x -> x * 2                      # Lambda
result = [1,2,3].map(x -> x ** 2)        # Higher-order
pipeline = data |> transform |> filter    # Pipeline operator
```

### Pattern Matching
```ruby
match response {
    Ok(data)           -> process(data)
    Err("timeout")     -> retry()
    Err(msg) if critical -> alert(msg)
    _                  -> log("unhandled")
}
```

### Classes & Inheritance
```ruby
class Animal {
    name: Str
    init(name) { self.name = name }
    def speak() { return "..." }
}

class Dog : Animal {
    def speak() { return "{self.name}: Woof!" }
    operator ==(other: Dog) -> Bool { return self.name == other.name }
}
```

### Parallel Execution
```ruby
parallel {
    users  = fetch_users()     # All three run
    orders = fetch_orders()    # concurrently —
    stats  = compute_stats()   # ~1x time, not 3x
}
```

### Error Handling
```ruby
result = read_file("config.json")?    # ? propagates errors
city = user?.address?.city ?? "Unknown"  # Optional chaining + nil coalescing

try {
    risky_operation()
} catch IOError as e {
    log("IO failed: {e}")
} finally {
    cleanup()
}
```

### Genetic Evolution
```ruby
genetic class Strategy {
    chromosome params {
        gene threshold: Float = 0.5 { range 0.0..1.0 }
        gene window: Int = 20 { range 5..200 }
    }
    fitness(data: Float) -> Float {
        return evaluate(self.threshold, self.window, data)
    }
}

best = evolve Strategy {
    population: 100, generations: 50,
    mutation_rate: 0.1, fitness on data: training_data
}
# Automatically finds optimal threshold and window!
```

### TTL Variables (Unique to Aether)
```ruby
# Variables that expire after a set time
ttl_set("auth_token", "jwt_abc123", 30.0)  # Expires in 30 seconds
print(ttl_get("auth_token"))                 # "jwt_abc123"
time_sleep(31.0)
print(ttl_get("auth_token"))                 # nil — expired!

# TTL Maps for caching
cache = ttl_map_new()
ttl_map_set(cache, "user:123", user_data, 300.0)  # 5-minute cache
```

### Python Interop
```ruby
use python.numpy as np
arr = np.array([1, 2, 3, 4, 5])
print(np.mean(arr))        # 3.0
print(np.linalg.det(mat))  # Full NumPy access from Aether
```

### Data Science in 5 Lines
```ruby
df = data_from_csv(fs_read("sales.csv"))
data_describe(df)                              # Print stats table
high_earners = data_where(df, "salary", ">", 80000)
viz_bar(data_column(df, "name"), data_column(df, "salary"), "Salaries")
fit = math_linear_regression(data_column(df, "x"), data_column(df, "y"))
```

---

## Built-in Standard Library

**37 modules, 133+ methods — all built into the binary. No package installs needed.**

<details>
<summary><strong>String</strong> — 50 methods</summary>

`len`, `upper`, `lower`, `trim`, `split`, `contains`, `starts_with`, `ends_with`, `replace`, `slice`, `chars`, `repeat`, `find`, `rfind`, `count`, `capitalize`, `title`, `is_empty`, `is_numeric`, `is_alpha`, `is_alphanumeric`, `is_whitespace`, `pad_left`, `pad_right`, `center`, `remove_prefix`, `remove_suffix`, `reverse`, `split_lines`, `split_whitespace`, `to_int`, `to_float`, `char_at`, `bytes`, `truncate`, `matches`, `insert_at`, `parse_int`, `parse_float`, and more
</details>

<details>
<summary><strong>List</strong> — 56 methods</summary>

`len`, `push`, `pop`, `first`, `last`, `get`, `map`, `filter`, `reduce`, `sort`, `sort_by`, `sort_desc`, `reverse`, `contains`, `index_of`, `find`, `find_index`, `take`, `drop`, `take_while`, `drop_while`, `compact`, `flatten`, `unique`, `dedup`, `partition`, `window`, `chunks`, `zip`, `zip_with`, `concat`, `interleave`, `rotate`, `enumerate`, `map_indexed`, `join`, `sum`, `min`, `max`, `min_by`, `max_by`, `mean`, `median`, `any`, `all`, `count`, `frequencies`, `flat_map`, `to_set`, `to_map`, `slice`, `sample`, `insert`, `remove`, `is_empty`, `is_sorted`
</details>

<details>
<summary><strong>Map</strong> — 15 methods &nbsp;|&nbsp; <strong>Set</strong> — 12 methods</summary>

**Map**: `len`, `get`, `get_or`, `set`, `remove`, `contains_key`, `keys`, `values`, `entries`, `merge`, `filter_keys`, `map_values`, `is_empty`, `to_list`, `invert`

**Set**: `len`, `contains`, `insert`, `add`, `remove`, `union`, `intersect`, `difference`, `is_empty`, `is_subset`, `is_superset`, `to_list`
</details>

| Module | What it provides | Python equivalent |
|--------|-----------------|-------------------|
| **math** | 30+ functions: trig, log, rounding, gcd, lcm, factorial, primes | `import math` |
| **stats** | mean, median, mode, std, percentile, z-scores, regression, correlation | `pip install numpy scipy` |
| **random** | int, float, bool, choice, shuffle, sample, uuid, hex | `import random` + `pip install uuid` |
| **regex** | match, find_all, replace, split | `import re` |
| **crypto** | sha256, sha512, md5 | `import hashlib` |
| **encoding** | base64, hex, url encode/decode | `import base64` |
| **json** | encode, decode, pretty | `import json` |
| **csv** | parse, encode, read, write | `import csv` |
| **fs** | read, write, append, delete, copy, mkdir, glob, 20+ functions | `import os, shutil, glob` |
| **http** | get, post, put, delete, patch, head + HTTP server | `pip install requests` + `pip install flask` |
| **time** | now, format, sleep, year/month/day, weekday, measure | `import datetime, time` |
| **term** | colors, bold, underline, progress bar, tables | `pip install colorama, tqdm` |
| **process** | exec, env vars, platform, arch, cpu count | `import subprocess, os` |
| **log** | debug, info, warn, error with colors and timestamps | `pip install logging` (stdlib but verbose) |
| **uuid** | v4 generation, validation | `pip install uuid` |
| **data** | DataFrame: load CSV, filter, sort, join, describe, print tables | `pip install pandas` |
| **viz** | Bar charts, line plots, histograms, scatter, heatmap, sparklines | `pip install matplotlib` |
| **tensor** | zeros, ones, random, matmul, transpose, reshape, sum, mean | `pip install numpy` |
| **concurrency** | Atomic counters, Channels, Mutex | `import threading` + manual |
| **ttl** | TTL variables, TTL maps (auto-expiring cache) | *No equivalent* |
| **collections** | Counter, deque, stack | `from collections import ...` |
| **iter** | chain, flatten, window, repeat, product, group_by | `import itertools` |

---

## CLI Reference

```bash
# Run & Execute
aether run <file.ae>          # Run a source file
aether run <file.ae> --vm     # Run using bytecode VM
aether run <file.ae> --watch  # Auto-restart on file changes
aether jit <file.ae>          # Compile via Cranelift JIT
aether repl                   # Interactive REPL
aether debug <file.ae>        # CLI debugger with breakpoints

# Project Management (Forge)
aether new <name>             # Create new project
aether init                   # Initialize in current directory
aether add <pkg>              # Add dependency
aether install                # Install dependencies
aether build                  # Type-check project
aether test                   # Run test files
aether fmt <file.ae>          # Format source code

# Tools
aether check <file.ae>        # Type-check with #strict mode
aether lsp                    # Language Server (VS Code)
```

---

## Architecture

```
Source (.ae) → Lexer → Parser → AST
                                 ↓
              ┌──────────────────┼──────────────────┐
              ↓                  ↓                   ↓
        Interpreter         Bytecode VM        Cranelift JIT
        (tree-walk)         (stack-based)      (native code)
                                                     ↓
                              ┌───────────────────────┼──────────┐
                              ↓                       ↓          ↓
                         CUDA PTX              WebGPU WGSL   ROCm HIP
```

| Component | Lines | Description |
|-----------|-------|-------------|
| Lexer + Parser | ~3,500 | Tokenizer + recursive descent with Pratt precedence |
| AST | ~900 | Complete node types for all language features |
| Interpreter | ~4,000 | Tree-walk with slot-indexed locals, inline arithmetic fast paths |
| Type Checker | ~200 | Gradual typing with `#strict` mode |
| Standard Library | ~8,000 | 37 modules, 133+ native functions |
| Bytecode VM | ~500 | Stack-based VM |
| GPU Codegen | ~300 | CUDA PTX, WebGPU WGSL, ROCm HIP |
| Tools | ~2,000 | REPL, debugger, LSP, DAP, Forge, watcher |
| **Total** | **~21,000** | **663 tests, 28 examples, 5000 fuzz runs** |

---

## Performance

Benchmarked against Python 3.10 on ARM64:

| Benchmark | Aether | Python | Status |
|-----------|--------|--------|--------|
| **Startup time** | 1ms | 11ms | **Aether 11x faster** |
| **String ops** | 4ms | 11ms | **Aether 2.8x faster** |
| **List ops 50K** | 14ms | 14ms | **Tied** |
| **Class ops 10K** | 22ms | 17ms | 1.3x (was 13x before optimization) |
| **Loop 1M** | 75ms | 60ms | 1.25x (was 3.1x before optimization) |
| **Fibonacci(30)** | 700ms | 150ms | 4.7x (was 152x before optimization) |

Key optimizations applied:
- Flat Vec environment with scope markers (replaced HashMap)
- Inline arithmetic fast paths for Int operations
- Slot-indexed locals with Rc-cached name resolution
- Method call push/pop instead of environment clone
- Range loop fast path (no Vec materialization)

---

## Examples

28 example programs in `examples/`:

| File | What it demonstrates |
|------|---------------------|
| `hello.ae` | Hello world, string interpolation |
| `basics.ae` | Variables, math, functions, loops, classes, lambdas |
| `control_flow.ae` | If/else, match, for/loop variants, break/next, guard |
| `error_handling.ae` | Result types, ? propagation, try/catch/finally |
| `pattern_matching.ae` | Literal, range, destructure, guard, enum patterns |
| `classes.ae` | OOP, inheritance, interfaces, structs, enums |
| `genetic_evolution.ae` | Genetic classes, evolve block, fitness optimization |
| `full_stack.ae` | Every major feature in one program |
| `stdlib_demo.ae` | All 16 core stdlib modules in action |
| `python_bridge.ae` | Call Python math, json, os from Aether |
| `python_numpy.ae` | NumPy arrays, matmul, linalg from Aether |
| `data_analysis.ae` | DataFrame: load, filter, sort, join, describe |
| `viz_demo.ae` | Bar charts, histograms, line plots, tables, sparklines |
| `math_demo.ae` | Statistics, regression, calculus, linear algebra, clustering |
| `http_client.ae` | HTTP GET/POST with real API calls |
| `concurrency_demo.ae` | Atomic counters, channels, mutex |
| `ttl_demo.ae` | TTL variables, auto-expiring cache |
| `apps/todo_api.ae` | Complete REST API server (~40 lines) |
| `apps/data_analyzer.ae` | CSV analysis pipeline with visualization |
| `apps/genetic_optimizer.ae` | Find function minimum via genetic evolution |

---

## VS Code Extension

Full IDE support in a separate repo: **[aether-vscode](https://github.com/Hattussa-IT-Solutions/aether-vscode)**

- IntelliSense (autocomplete, hover, go-to-definition)
- Live diagnostics (parse + type errors as you type)
- Debugging (breakpoints, stepping, variable inspection)
- Run button (Ctrl+Shift+R) and Debug button (F5)
- Syntax highlighting, 11 code snippets

---

## Installation

**Linux / macOS:**
```bash
curl -sSf https://raw.githubusercontent.com/Hattussa-IT-Solutions/Aether/main/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Hattussa-IT-Solutions/Aether/main/install.ps1 | iex
```

**From source:**
```bash
git clone https://github.com/Hattussa-IT-Solutions/Aether.git
cd Aether && cargo build --release
```

**Docker:**
```bash
docker build -t aether .
docker run --rm -v $(pwd):/work aether run /work/hello.ae
```

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
git clone https://github.com/Hattussa-IT-Solutions/Aether.git
cd Aether
cargo build && cargo test
cargo run -- run examples/hello.ae
```

---

## License

Apache License 2.0 — see [LICENSE](LICENSE).

---

<p align="center">
  Built with Rust by <a href="https://github.com/Hattussa-IT-Solutions">Hattussa IT Solutions</a>
</p>
