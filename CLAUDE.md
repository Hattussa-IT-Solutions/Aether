# AETHER PROGRAMMING LANGUAGE — COMPILER PROJECT

## Mission
Build the complete Aether programming language: compiler, runtime, GPU backends, standard library, package manager, VS Code extension, and tooling. Everything in Rust (compiler) and TypeScript (IDE extension).

## Architecture Overview
```
aether/
├── Cargo.toml
├── src/
│   ├── main.rs                    # CLI entry: aether run/build/repl/test/new
│   ├── lib.rs                     # Library root
│   ├── lexer/
│   │   ├── mod.rs
│   │   ├── tokens.rs              # Token enum (all types)
│   │   └── scanner.rs             # Tokenizer/lexer
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── ast.rs                 # AST node types
│   │   ├── parser.rs              # Recursive descent parser
│   │   └── precedence.rs          # Operator precedence (Pratt parsing)
│   ├── types/
│   │   ├── mod.rs
│   │   ├── checker.rs             # Type checking pass
│   │   └── inference.rs           # Hindley-Milner type inference
│   ├── interpreter/
│   │   ├── mod.rs
│   │   ├── environment.rs         # Variable scoping (stack of hashmaps)
│   │   ├── values.rs              # Runtime value types
│   │   ├── eval.rs                # Expression evaluator
│   │   ├── exec.rs                # Statement executor
│   │   └── parallel.rs            # parallel {} block runtime
│   ├── compiler/
│   │   ├── mod.rs
│   │   ├── bytecode.rs            # Bytecode instruction set
│   │   ├── compiler.rs            # AST -> Bytecode compiler
│   │   └── vm.rs                  # Stack-based bytecode VM
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── cranelift.rs           # Bytecode -> Cranelift -> native
│   │   ├── cuda.rs                # @gpu -> CUDA PTX text generation
│   │   ├── wgsl.rs                # @gpu -> WebGPU WGSL generation
│   │   └── hip.rs                 # @gpu -> ROCm HIP generation
│   ├── stdlib/
│   │   ├── mod.rs                 # Standard library registry
│   │   ├── string.rs              # Str methods
│   │   ├── list.rs                # List/Array methods
│   │   ├── map.rs                 # Map/Dict methods
│   │   ├── set.rs                 # Set methods
│   │   ├── math.rs                # Math functions + constants
│   │   ├── io.rs                  # File I/O
│   │   ├── net.rs                 # HTTP client/server
│   │   ├── json.rs                # JSON encode/decode
│   │   ├── time.rs                # DateTime, Duration
│   │   └── tensor.rs              # Tensor type + operations
│   ├── bridge/
│   │   ├── mod.rs
│   │   └── python.rs              # CPython embedding via pyo3
│   ├── repl/
│   │   └── mod.rs                 # Interactive REPL
│   ├── diagnostics/
│   │   └── mod.rs                 # Error messages + suggestions
│   └── forge/
│       ├── mod.rs                 # Package manager
│       ├── toml.rs                # aether.toml parser
│       └── resolver.rs            # Dependency resolution
├── tests/
│   ├── lexer_tests.rs
│   ├── parser_tests.rs
│   ├── type_tests.rs
│   ├── interpreter_tests.rs
│   ├── parallel_tests.rs
│   ├── oop_tests.rs
│   ├── genetic_tests.rs
│   └── gpu_tests.rs
├── examples/
│   ├── hello.ae
│   ├── parallel_demo.ae
│   ├── pattern_matching.ae
│   ├── classes.ae
│   ├── genetic_evolution.ae
│   ├── gpu_tensor.ae
│   ├── full_stack.ae
│   └── type_safety.ae
└── aether-vscode/                 # VS Code extension (TypeScript)
    ├── package.json
    ├── syntaxes/aether.tmLanguage.json
    └── src/extension.ts
```

## Cargo.toml Dependencies
```toml
[package]
name = "aether"
version = "0.1.0"
edition = "2021"

[dependencies]
cranelift-codegen = "0.105"
cranelift-frontend = "0.105"
cranelift-module = "0.105"
cranelift-jit = "0.105"
cranelift-native = "0.105"
tokio = { version = "1", features = ["full"] }
rayon = "1.8"
rustyline = "14"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

[dev-dependencies]
# pyo3 = { version = "0.21", features = ["auto-initialize"] }  # enable when Python bridge needed
```

---

## COMPLETE SYNTAX SPECIFICATION

### Blocks and structure
- Braces `{ }` for ALL blocks (functions, classes, if, loops, etc.)
- NO semicolons (newline = statement terminator)
- NO parentheses around conditions: `if x > 10 {` not `if (x > 10) {`
- `//` for line comments, `/* */` for block comments
- File extension: `.ae`

### Variables
```
x = 5                    // no keyword, mutable, type inferred
let x = 5                // immutable (cannot reassign)
const MAX = 100           // compile-time constant
x: Int = 5               // explicit type annotation

#strict                   // at top of file: requires let/const + types
let x: Int = 5            // required in strict mode
```

### Primitive types
```
Int       // 64-bit signed integer (default)
Float     // 64-bit IEEE 754 (default)
Num       // auto-promotes between Int and Float
Bool      // true, false
Str       // UTF-8 immutable string
Char      // single Unicode character ('A')
Byte      // unsigned 8-bit (0-255)
nil       // absence of value

// Sized variants
Int8, Int16, Int32, Int64
UInt8, UInt16, UInt32, UInt64
Float32, Float64
```

### Generic types (shorthand + standard)
```
Str[]                    // List<Str> — array syntax
Int[]                    // List<Int>
{Str: Int}               // Map<Str, Int> — JSON-like
User?                    // Optional<User> — nullable
Set<Int>                 // Set
Result<User, Error>      // Result
Tensor<Float32>          // Tensor
```

### Operators (ALL of these must be lexed)
```
// Arithmetic
+  -  *  /  %  **

// Comparison
==  !=  <  >  <=  >=

// Logical
&&  ||  !
and  or  not           // aliases for && || !

// Bitwise
&  |  ^  ~  <<  >>

// Assignment
=  +=  -=  *=  /=  %=  **=  &=  |=  ^=  <<=  >>=

// Special
->                      // return type, lambda, match arm
?.                      // optional chaining
??                      // nil coalescing
?                       // error propagation (postfix)
|>                      // pipeline operator
..                      // exclusive range
..=                     // inclusive range
=>                      // computed property shorthand

// Brackets
{  }  (  )  [  ]

// Punctuation
:  ,  .  @  #  ;(ignored/optional)
```

### Strings
```
"Hello {name}"           // auto-interpolation (no f-prefix)
"Escaped: \n \t \\ \""   // standard escapes
"Literal brace: {{}}"    // {{ = literal {
r"raw\nstring"           // raw string (no escapes)
"""                      // multiline (indentation auto-trimmed)
    multi
    line
"""
'A'                      // Char literal
```

### Functions
```
def name(params) { }                  // basic function
def name(a: Int, b: Int) -> Int { }   // typed with return
def double(x: Num) -> Num = x * 2     // one-liner (expression body)
async def fetch(url: Str) -> Data { }  // async function

// Default parameters (use = in definition)
def connect(host: Str, port: Int = 8080) { }

// Named parameters at call site (use : )
connect("example.com", port: 9090)

// Variadic
def sum(*nums: Num) -> Num { }
def create(**fields) { }

// Lambdas (-> arrow)
x -> x * 2
(a, b) -> a + b

// Trailing closure
items.filter { x -> x > 0 }
items.map { item ->
    process(item)
}

// Decorators
@gpu
@test
@cached(ttl: 60)
@experiment("name")
@deprecated("use X instead")
```

### Control flow
```
// if / else if / else
if condition {
} else if condition {
} else {
}

// if as expression
val = if x > 0 then "pos" else "neg"

// guard (early return)
guard let value = optional else {
    return Err("missing")
}

// nil-safe
user?.address?.city          // optional chaining
name ?? "Anonymous"          // nil coalescing

// if let (unwrap optional)
if let user = maybe_user {
    print(user.name)
}
```

### Pattern matching (match)
```
match value {
    "ok"       -> handle_ok()           // literal
    42         -> handle_42()           // number
    1..10      -> handle_range()        // range
    _          -> handle_default()      // wildcard
}

// Destructuring
match result {
    Ok(data)           -> process(data)
    Err("timeout")     -> retry()
    Err(msg) if msg.len() > 50 -> log(msg)   // with guard
    Err(msg)           -> fail(msg)
}

// Enums
match shape {
    .Circle(r)    -> PI * r ** 2
    .Rect(w, h)   -> w * h
}

// match is an expression (returns value)
grade = match score {
    90..100 -> "A"
    80..89  -> "B"
    _       -> "F"
}
```

### Loops
```
// for: iteration
for item in list { }                    // for-each
for i, item in list { }                 // enumerate (BUILT IN)
for i in 0..10 { }                      // range (exclusive end)
for i in 1..=10 { }                     // range (inclusive end)
for i in 0..100 step 2 { }             // stride
for i in 10..0 { }                      // reverse
for key, val in map { }                 // dict iteration
for (name, age) in tuples { }          // destructure

// loop: repetition
loop 5 times { }                        // counted (no variable)
loop while condition { }                // while
loop { }                                // infinite
loop {                                  // do-until
    input = read()
} until input == "quit"

// Control
break                                   // exit loop
next                                    // skip iteration (NOT continue)
next if x == 0                          // one-liner guard
break:label                             // named break for nested loops

for:outer row in matrix {
    for col in row {
        if col == target { break:outer }
    }
}

// Parallel loop
for item in items |parallel| { }
for item in items |parallel: 10| { }   // with concurrency limit

// Comprehensions
[x ** 2 for x in 0..10]               // list
[x for x in data if x > 0]            // filtered
{k: v for (k, v) in pairs}            // map
```

### OOP: class, struct, enum, interface
```
// Class (reference type, heap, ARC)
class Dog {
    name: Str
    breed: Str
    age: Int = 0

    init(name: Str, breed: Str) {
        self.name = name
        self.breed = breed
    }

    def bark() { print("{self.name}: Woof!") }
}

// Struct (value type, stack, copy-on-assign)
struct Point {
    x: Float
    y: Float
    def magnitude() -> Float = sqrt(x ** 2 + y ** 2)
}

// Enum (algebraic data type, with associated values)
enum Shape {
    Circle(radius: Float)
    Rect(width: Float, height: Float)

    def area() -> Float {
        match self {
            .Circle(r)    -> PI * r ** 2
            .Rect(w, h)   -> w * h
        }
    }
}

// Interface (behavior contract)
interface Serializable {
    def to_json() -> Str
}

// Implementation
class User impl Serializable {
    name: Str
    def to_json() -> Str { return json.encode(self) }
}

// Inheritance (single, with :)
class Admin : User {
    role: Str = "admin"
}

// Access modifiers
pub name: Str          // public (default)
priv secret: Str       // private
prot id: Int           // protected
readonly created: Str  // read-only

// Computed property
area: Float => width * height

// Observed property
value: Int = 0 {
    did_change(old, new) { log("changed") }
}

// Lazy property
lazy data: Data => expensive_load()

// Static
static def create() -> Self { }
static let INSTANCE: Self = Self()

// Operator overloading
operator +(other: Self) -> Self { }
operator ==(other: Self) -> Bool { }

// Type alias
type UserID = Int
type Callback = (Int, Str) -> Bool

// Destructor
deinit { cleanup() }
```

### Error handling
```
// Result type
def parse(s: Str) -> Result<Int, ParseError> { }

// ? operator (propagate error)
data = read_file("x.csv")?

// try / catch / finally
try {
    risky()
} catch IOError as e {
    handle(e)
} catch any as e {
    log(e)
} finally {
    cleanup()
}

// throw
throw CustomError("message")

// Optional chaining + nil coalescing
city = user?.address?.city ?? "Unknown"

// Custom error
class AppError : Error {
    code: Int
    message: Str
}
```

### Parallel blocks (UNIQUE TO AETHER)
```
// All tasks run simultaneously. Block waits for all.
parallel {
    users  = fetch_users()
    orders = fetch_orders()
    stats  = fetch_stats()
}

// Dependencies
parallel {
    data   = load()
    config = load_config()
    model  = after(data, config) { train(data, config) }
    report = after(model) { evaluate(model) }
}

// Race (first wins, others cancel)
result = parallel.race {
    fast = cache_lookup(id)
    slow = db_query(id)
}

// Timeout
parallel(timeout: 5.seconds) { }

// Max concurrency
parallel(max: 10) { }
```

### Modules
```
use fs                    // import module
use net.http              // nested module
use json as j             // alias
use python.torch as torch // Python bridge

mod utils {               // define module
    pub def helper() { }
}
```

---

## 15 NEW OOP CONCEPTS (implement ALL of these)

### 1. reactive — auto-recomputing properties
```
class Cart {
    items: Product[] = []
    reactive total: Float = items.sum(p -> p.price)
    reactive count: Int = items.len()
    // total and count auto-update when items changes
}
```

### 2. temporal — time-aware state (ring buffer history)
```
class Sensor {
    temporal(keep: 100) temperature: Float = 20.0

    def analyze() {
        avg = temperature.history.mean()
        trend = temperature.history.trend()  // .rising/.falling/.stable
        prev = temperature.previous
    }
}
```

### 3. mutation — controlled state changes
```
class Account {
    mutation(tracked, undoable: 10) balance: Float = 0.0 {
        constrain 0.0..1000000.0         // compile-time range
        validate { value >= 0.0 }         // runtime check
        transform { round(value, 2) }     // auto-modify before store
        redact                            // hide in logs/debug
        rule only_increase_by: deposit()  // only specific methods can increase
        rule only_decrease_by: withdraw() // only specific methods can decrease
    }
}

// Tracked: balance.mutations gives full audit log
// Undoable: balance.rollback(), balance.forward()
// Atomic transactions:
mutation.atomic {
    from.withdraw(100.0)
    to.deposit(100.0)
    // all succeed or all revert
}

// Freeze
config.freeze()      // no more changes
config.unfreeze()    // allow changes again
config.frozen { }    // frozen within scope only
let permanent = config.to_frozen()  // permanent, type changes
```

### 4. weave — compile-time behavior injection
```
weave Logged {
    before { log.info("{method.name} called") }
    after  { log.info("{method.name}: done") }
}

weave Cached(ttl: Int = 60) {
    before { if let cached = cache.get(method.cache_key) { return cached } }
    after  { cache.set(method.cache_key, result, ttl: ttl) }
}

class UserService with Logged, Cached(ttl: 120) {
    def getUser(id: Int) -> User { return db.find(id) }
}
```

### 5. bond — bidirectional relationships
```
class Student { bond courses: Course[] via students }
class Course  { bond students: Student[] via courses }
// alice.courses.push(math) -> math.students auto-includes alice
```

### 6. face — object projections
```
class User {
    name: Str
    email: Str
    priv ssn: Str
    face public { show name }
    face admin  { show name, email, ssn }
}
user.as(.public)  // only name accessible, compile-time enforced
```

### 7. extend — add methods to any type
```
extend Int {
    def is_even() -> Bool = self % 2 == 0
}
5.is_even()  // false

extend T[] where T: Numeric {
    def mean() -> Float = self.sum() / self.len()
}
```

### 8. delegate — auto-forward methods
```
class UniqueList<T: Equatable> {
    priv inner: T[] = []
    delegate inner: T[]          // forwards ALL list methods
    override def push(item: T) {
        if !inner.contains(item) { inner.push(item) }
    }
}
```

### 9. select / exclude — selective inheritance
```
class Penguin : Animal select(swim, run, breathe) {
    // fly() and climb() DO NOT EXIST on Penguin
}
class Penguin : Animal exclude(fly, climb) {
    // same effect
}
```

### 10. morph — compile-time conditional methods
```
class Logger {
    morph def log(msg: Str) {
        when target == .debug   { print("[DEBUG] {msg}") }
        when target == .release { metrics.count("log") }
        when target == .embedded { /* nothing — zero overhead */ }
    }
}
```

### 11-15. evolving, capabilities, dimensional types
```
// evolving: runtime strategy adaptation
evolving strategy: Strategy = .default {
    every 100 calls { evolve_to(best_performing) }
}

// capabilities: compile-time security
class Processor with capabilities(read_file, cpu_compute) {
    // CANNOT access network — compile error if it tries
}

// dimensional types
distance: Float.m = 100.0       // meters
time: Float.s = 9.58            // seconds
speed = distance / time          // auto-derived: Float.m_per_s
```

---

## GENETIC CLASSES

```
genetic class NeuralArch {
    chromosome layers {
        gene hidden_size: Int = 128 { range 32..1024, step 32 }
        gene num_layers: Int = 4 { range 1..24 }
        gene activation: Activation = .relu { options [.relu, .gelu, .silu] }
        gene dropout: Float = 0.1 { range 0.0..0.5, step 0.05 }
    }
    chromosome training {
        gene lr: Float = 0.001 { range 1e-5..0.1, scale .log }
        gene batch_size: Int = 32 { options [8, 16, 32, 64, 128] }
        gene optimizer: Optimizer = .adam { options [.adam, .sgd, .adamw] }
    }

    fitness(data: Dataset) -> Float {
        model = self.build()
        model.train(data.train, epochs: 5)
        return model.evaluate(data.test).accuracy
    }
}

// Mutation operations
mutant = original.mutate(.point)              // change one gene
mutant = original.mutate(.chromosome, target: .layers)
child  = crossover(parent_a, parent_b)        // chromosome swap
child  = breed(parent_a, parent_b, mutation_rate: 0.1)

// Evolution
best = evolve NeuralArch {
    population: 100
    generations: 50
    mutation_rate: 0.1
    crossover_rate: 0.7
    selection: .tournament(size: 5)
    elitism: 0.1
    fitness on data: my_dataset
}
```

---

## GPU BACKENDS

### @gpu decorator
Compiles function body to GPU code. Auto-detects hardware.

### CUDA PTX (NVIDIA)
Generate PTX assembly as text string. Compile with nvcc or load via cuModuleLoadData. Target: compute_70+ for GH200.

### WebGPU WGSL (Browser / any GPU)
Generate WGSL compute shader text. Runs in any modern browser without drivers.

### ROCm HIP (AMD)
Generate HIP source code. Nearly identical to CUDA syntax but targets AMD GPUs.

### device() blocks
```
device(.gpu) { tensor_ops() }
device(.cpu) { io_ops() }
device(.quantum) { quantum_ops() }
```

---

## BUILD ORDER (follow this exactly)
1. Cargo skeleton + all module stubs
2. Lexer (ALL tokens, keywords, operators, literals)
3. AST types (EVERY node type for the full language)
4. Parser (recursive descent with Pratt precedence for expressions)
5. Interpreter core (variables, arithmetic, strings, print, functions)
6. Interpreter control flow (if/else, match, for, loop)
7. Interpreter OOP (class, struct, enum, interface, impl, inheritance)
8. Standard library (Str, List, Map, Set, Math, basic I/O)
9. Error handling (try/catch, ?, ?., ??, Result, throw)
10. parallel {} blocks (Tokio runtime)
11. REPL (rustyline)
12. Type checker + inference + #strict mode
13. New OOP concepts (reactive, temporal, mutation, weave, bond, face, delegate, select, morph, extend)
14. Genetic classes (genetic class, gene, chromosome, evolve)
15. Bytecode compiler + stack VM
16. Cranelift native code generation
17. Tensor library + basic autodiff
18. GPU: CUDA PTX generation (test on GH200!)
19. GPU: WebGPU WGSL generation
20. Python bridge (pyo3)
21. Forge package manager
22. Error diagnostics polish
23. Comprehensive test suite
24. Example programs + benchmarks
25. VS Code extension

## TESTING RULES
- After EVERY step, run `cargo build` and `cargo test`
- Write tests BEFORE implementing features when possible
- Create `.ae` example files and verify they produce correct output
- For GPU: compile PTX with `nvcc`, verify execution on GH200
- Every parser rule needs at least 2 test cases
- Every interpreter feature needs at least 3 test cases
- Run `cargo clippy` periodically for Rust best practices

## CODE QUALITY RULES
- Use `Rc<RefCell<T>>` for interpreter values (avoids lifetime complexity)
- Clone liberally in v1 — optimize later
- Every public function needs a doc comment
- Error messages must include source file, line number, column, and a suggestion
- Use `thiserror` crate for error types
- Use `miette` or `ariadne` crate for beautiful error diagnostics
