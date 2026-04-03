# AETHER BUILD PROMPTS — Paste these into Claude Code one at a time
# Each prompt is one session. Wait for it to complete before starting the next.
# ═══════════════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════
SESSION 1: Project Skeleton + Lexer + AST
═══════════════════════════════════════════════════════════════

Read CLAUDE.md completely. This is the full specification for the Aether programming language.

Step 1: Create the complete Cargo project with ALL module files and the directory structure exactly as specified in CLAUDE.md. Use the Cargo.toml dependencies listed there. Create every directory and mod.rs file even if the module is initially empty (just put // TODO comments).

Step 2: Implement the COMPLETE lexer/tokenizer in src/lexer/. This must handle:
- All keywords: def, let, const, if, else, match, guard, for, in, loop, times, while, until, step, break, next, return, class, struct, enum, interface, impl, self, super, init, deinit, pub, priv, prot, readonly, static, lazy, async, await, try, catch, finally, throw, use, mod, as, type, parallel, after, device, model, agent, pipeline, reactive, temporal, mutation, evolving, genetic, gene, chromosome, fitness, crossover, breed, evolve, weave, bond, face, extend, delegate, select, exclude, morph, true, false, nil, and, or, not, then, operator, override, where
- All operators: + - * / % ** = += -= *= /= %= **= == != < > <= >= && || ! & | ^ ~ << >> &= |= ^= <<= >>= -> ?. ?? ? |> .. ..= => . , : @ # ( ) [ ] { }
- String literals with interpolation detection: "Hello {name}" — tokenize the parts
- Multiline strings: """..."""
- Raw strings: r"..."
- Char literals: 'A'
- Number literals: integers (42, 1_000), floats (3.14, 1.0e10), hex (0xFF), binary (0b1010)
- Comments: // line and /* block */
- Directives: #strict, #test
- Decorators: @gpu, @test, @cached, @experiment, @deprecated (and any @identifier)
- Proper source location tracking (line, column) on every token

Step 3: Define ALL AST node types in src/parser/ast.rs. Every expression, statement, and declaration type needed for the full language. Include all OOP constructs, parallel blocks, genetic classes, match patterns, etc. Use Rust enums with Box<T> for recursive types.

Step 4: Write comprehensive tests in tests/lexer_tests.rs covering every token type, edge cases (nested strings, emoji in strings, large numbers, chained operators).

Run cargo build and cargo test. Fix ALL errors until both pass with zero failures. Do not stop until the project compiles cleanly and all tests pass.


═══════════════════════════════════════════════════════════════
SESSION 2: Parser (All Syntax)
═══════════════════════════════════════════════════════════════

Read the existing codebase. You already have the lexer and AST types from Session 1.

Implement the COMPLETE recursive descent parser in src/parser/parser.rs with Pratt parsing for expression precedence in src/parser/precedence.rs.

The parser must handle ALL Aether syntax:

Expressions:
- Literals (int, float, string with interpolation, bool, nil, char)
- Identifiers and qualified names (module.name)
- Binary operations (all arithmetic, comparison, logical, bitwise, range .., ..=)
- Unary operations (-, !, ~, not)
- Power operator ** (right-associative)
- Function calls with named parameters: func(a, b, key: value)
- Method calls: obj.method(args)
- Index access: list[0], map["key"]
- Optional chaining: obj?.field
- Nil coalescing: x ?? default
- Error propagation: expr?
- Pipeline: a |> b |> c
- Lambda: x -> x * 2, (a, b) -> a + b
- If expression: if cond then a else b
- Match expression (returns value)
- List literal: [1, 2, 3]
- Map literal: {"key": value}
- Set literal: {1, 2, 3}  (disambiguate from map by checking for : )
- Tuple: (a, b, c)
- Range: 0..10, 1..=10, 0..100 step 2
- Comprehension: [expr for x in iter if cond]
- Computed property: field: Type => expr

Statements:
- Variable declaration (with and without keyword/type)
- Assignment (=, +=, -=, *=, etc.)
- Expression statement
- if / else if / else blocks
- guard let ... else { }
- if let ... { } else { }
- match block with pattern arms (literal, range, destructure, wildcard, guard)
- for loops (single var, two var enumerate, range, step, dict, destructure)
- loop (times, while, infinite, until)
- break, next, break:label, next if condition
- return (with optional value)
- throw expression
- try / catch / finally
- parallel { } block (each statement is a task)
- after(deps) { expr } inside parallel
- parallel.race { }
- parallel(timeout: expr) { }
- mutation.atomic { } block
- device(.gpu) { }, device(.cpu) { }, device(.quantum) { }

Declarations:
- def function (params with types/defaults, return type, body or = expr)
- async def
- class (fields, init, deinit, methods, computed/observed/lazy props, operator overloading, static)
- struct (fields, methods, computed props)
- enum (simple variants, variants with associated values, methods)
- interface (method signatures, default methods)
- class with impl Interface
- class with inheritance (: Parent)
- class with select/exclude
- class with capabilities
- class with weave: "class X with Logged, Cached { }"
- weave definition (before/after/around blocks)
- bond declarations inside class
- face declarations inside class
- extend blocks
- delegate declarations
- morph methods
- reactive/temporal/mutation property modifiers
- evolving property
- genetic class with chromosome/gene/fitness
- mod block
- use statement (with as alias)
- type alias
- Decorators (@gpu, @test, etc.) on functions and classes
- #strict directive

Write comprehensive parser tests in tests/parser_tests.rs. Test every construct.

Create examples/hello.ae:
```
print("Hello from Aether!")
```

Create examples/syntax_test.ae with one example of every major syntax construct.

Run cargo build and cargo test. Fix ALL errors. Do not stop until everything compiles and passes.


═══════════════════════════════════════════════════════════════
SESSION 3: Interpreter Core
═══════════════════════════════════════════════════════════════

Read the existing codebase. You have lexer, AST, and parser.

Implement the tree-walk interpreter in src/interpreter/:

1. values.rs — Runtime value types:
   - Integer, Float, Num, Bool, String, Char, Byte, Nil
   - List (Vec<Value>), Map (HashMap), Set (HashSet), Tuple
   - Function (params, body, closure environment)
   - Class instance, Struct instance, Enum variant
   - NativeFunction (for stdlib)
   - Use Rc<RefCell<T>> for shared mutable references

2. environment.rs — Scoping:
   - Stack of HashMap<String, Value>
   - push_scope(), pop_scope(), get(), set(), define()
   - Closures capture their environment

3. eval.rs — Expression evaluator:
   - All arithmetic (with Num auto-promotion: Int op Float -> Float)
   - String concatenation and interpolation
   - Comparison operators
   - Logical operators (&& || ! and or not) with short-circuit
   - Bitwise operators
   - List/Map/Set literals
   - Index access (list[i], map[key])
   - Function calls (positional + named args)
   - Method calls (obj.method())
   - Lambda creation and invocation
   - Optional chaining (?.) — returns nil if any part is nil
   - Nil coalescing (??) — return right if left is nil
   - Error propagation (?) — return Err early if Result is Err
   - Pipeline (|>) — a |> f = f(a)
   - Range objects (0..10, 1..=10)
   - Match expression
   - If expression (if cond then a else b)

4. exec.rs — Statement executor:
   - Variable declaration and assignment
   - print() built-in function
   - if / else if / else
   - match with all pattern types
   - for loops (for-each, enumerate, range, step, dict, destructure)
   - loop (times, while, infinite, until)
   - break, next, break:label
   - return
   - try / catch / finally / throw
   - Function definitions
   - Class definitions with init, methods, properties
   - Struct definitions
   - Enum definitions with associated values
   - Interface checking (verify impl has required methods)
   - Inheritance (method lookup chain: self -> parent -> grandparent)
   - use statements (basic module loading)

5. Create the CLI in main.rs:
   - `aether run <file.ae>` — parse and interpret
   - `aether repl` — start REPL (placeholder for now)
   - `aether --version` — print version

Test with examples/hello.ae:
```
print("Hello from Aether!")
name = "World"
print("Hello, {name}!")
```

Create examples/basics.ae testing variables, math, strings, if/else, loops, functions, classes.

Run cargo build. Then run: cargo run -- run examples/hello.ae
Verify it prints correctly. Fix all bugs until examples run correctly.


═══════════════════════════════════════════════════════════════
SESSION 4: Control Flow + Match + Error Handling
═══════════════════════════════════════════════════════════════

Read the existing codebase.

Ensure ALL control flow works correctly with thorough testing:

1. Pattern matching: literal patterns, range patterns, destructuring (Ok(val), Err(msg), enum variants, tuples), wildcard _, guard conditions (if expr after pattern), exhaustiveness checking for enums.

2. Loops: for with enumerate (for i, item in list), range with step, loop N times, loop while, loop-until, break:label for nested loops, next if condition.

3. Error handling: Result<T,E> type, Ok() and Err() constructors, ? operator that returns early on Err, try/catch/finally with type-specific catches, throw, custom error classes.

4. Guard statements: guard let value = expr else { return/throw/break }

Create examples/control_flow.ae demonstrating every control flow feature.
Create examples/error_handling.ae demonstrating Result, ?, try/catch.
Create examples/pattern_matching.ae with complex match examples.

Write tests in tests/interpreter_tests.rs for every edge case.
Run cargo test. Fix everything.


═══════════════════════════════════════════════════════════════
SESSION 5: OOP + Standard Library
═══════════════════════════════════════════════════════════════

Read the existing codebase.

1. Complete OOP implementation:
   - Class: fields with defaults, init constructor, methods, self reference, computed properties (=> syntax), observed properties (did_change), lazy properties, static methods/properties, deinit
   - Struct: value semantics (copy on assign), all features of class minus inheritance
   - Enum: simple variants, variants with associated values, methods on enums, pattern matching on .Variant(fields)
   - Interface: method signatures, default implementations, check conformance at class definition
   - Inheritance: single inheritance with :, super calls, method override, constructor chaining
   - Operator overloading: operator +, -, *, /, ==, !=, <, >, [], etc.
   - Type alias: type UserID = Int
   - Access modifiers: pub (default), priv (same class), prot (class + subclasses)

2. Standard library (implement as native functions registered in the interpreter):
   - Str: len(), upper(), lower(), trim(), split(sep), join(list), contains(sub), starts_with(), ends_with(), replace(old,new), slice(start,end), chars(), repeat(n), parse_int(), parse_float()
   - List (T[]): len(), push(item), pop(), insert(i,item), remove(i), first(), last(), map(fn), filter(fn), reduce(init,fn), sort(), reverse(), contains(item), index_of(item), flat_map(fn), zip(other), chunks(n), any(fn), all(fn), sum(), min(), max(), unique()
   - Map ({K:V}): len(), get(key), set(key,val), remove(key), contains_key(key), keys(), values(), entries()
   - Set (Set<T>): len(), insert(item), remove(item), contains(item), union(other), intersect(other), difference(other)
   - Math: sin, cos, tan, asin, acos, atan, atan2, sqrt, cbrt, pow, log, log2, log10, exp, abs, min, max, clamp, round, floor, ceil, trunc, PI, E, TAU, INF, random(), random(min,max)
   - IO: print(args...), input(prompt), fs.read(path), fs.write(path,content), fs.exists(path), fs.lines(path)

Create examples/classes.ae, examples/collections.ae, examples/math_demo.ae.
Run all tests and examples.


═══════════════════════════════════════════════════════════════
SESSION 6: Parallel Blocks + REPL
═══════════════════════════════════════════════════════════════

Read the existing codebase.

1. Implement parallel { } blocks in src/interpreter/parallel.rs using Tokio:
   - Each statement in a parallel block spawns as a Tokio task
   - Block waits for all tasks to complete (tokio::join!)
   - after(task1, task2) { expr } — waits for named tasks before executing
   - parallel.race { } — first completed task wins, cancel others (tokio::select!)
   - parallel(timeout: duration) { } — fail if not done in time
   - Parallel for loops: for x in items |parallel| { } using Rayon
   - for x in items |parallel: N| { } with concurrency limit
   - Return values from parallel tasks back to the outer scope

2. Implement mutation.atomic { } blocks:
   - Snapshot all mutation(tracked) values at block entry
   - If any statement throws/fails, restore all snapshots
   - Nested atomic blocks create savepoints

3. Implement REPL in src/repl/mod.rs using rustyline:
   - Interactive prompt with line editing
   - Multi-line input (detect unclosed braces)
   - History (up/down arrows)
   - Print expression results automatically
   - Handle errors gracefully (show error, continue)
   - :help, :quit, :clear commands

4. Add time module to stdlib:
   - time.now() -> Timestamp
   - time.since(start) -> Duration
   - time.sleep(duration)
   - Duration types: N.seconds, N.milliseconds

Create examples/parallel_demo.ae:
```
use time
t1 = time.now()
parallel {
    a = { time.sleep(1.seconds); "A done" }
    b = { time.sleep(1.seconds); "B done" }
    c = { time.sleep(1.seconds); "C done" }
}
print("Parallel: {time.since(t1)}")  // ~1 second, not 3
```

Run it and verify timing. Run all tests.


═══════════════════════════════════════════════════════════════
SESSION 7: Type Checker + Diagnostics
═══════════════════════════════════════════════════════════════

Read the existing codebase.

Implement the type checker in src/types/:

1. Type inference: infer types from assignments, function returns, expressions
2. Type checking: verify assignments, function arguments, return types match
3. Gradual typing: in default mode, allow missing types (skip checking). In #strict mode, require all types.
4. Generic type checking: verify Str[] contains only strings, {Str: Int} has correct key/value types
5. Nullable checking: T? can be nil, T cannot
6. Result checking: Result<T,E> must be handled (used with ?, try/catch, or match)
7. Enum exhaustiveness: match on enum must cover all variants
8. Interface conformance: class impl Interface must have all required methods
9. Operator type checking: + on Int+Int=Int, Int+Float=Float, Str+Str=Str, etc.

Implement beautiful error diagnostics in src/diagnostics/:
- Show source file, line number, column number
- Underline the error location with ^^^
- Show the error message in red
- Show suggestion in blue ("did you mean X?")
- For type errors: show expected vs actual type
- Use ariadne or miette crate for formatting

Create examples/type_errors.ae (intentional errors to test diagnostics).
Run all tests.


═══════════════════════════════════════════════════════════════
SESSION 8: New OOP Part 1 (Property Modifiers)
═══════════════════════════════════════════════════════════════

Read the existing codebase.

Implement the first set of new OOP concepts — property modifiers:

1. reactive properties:
   - Track dependency graph (which properties does this reactive depend on?)
   - When a dependency changes, recompute the reactive value
   - Inject notification calls at every assignment to a dependency

2. temporal(keep: N) properties:
   - Wrap property in a ring buffer of size N
   - Every assignment pushes old value to buffer
   - .history returns the buffer contents
   - .previous returns last value
   - .history.mean(), .history.trend() (rising/falling/stable)
   - .at(N.steps_ago) or .at(N.seconds_ago)

3. mutation system:
   - mutation(tracked): log every change with timestamp, old/new value, method name
   - mutation(undoable: N): ring buffer + rollback()/forward()
   - constrain range: compile-time check that values stay in range
   - validate { expr }: runtime check before every assignment
   - transform { expr }: modify value before storing
   - redact: override display/debug to show [REDACTED]
   - rule only_increase_by / only_decrease_by: restrict which methods can change value
   - .freeze() / .unfreeze() / .frozen { } scope
   - mutation.atomic { } (already done in Session 6)

4. evolving properties:
   - Property that changes strategy based on runtime evidence
   - every N calls { } evaluation block
   - evolve_to(new_strategy) switches behavior

Create examples/reactive_demo.ae, examples/mutation_demo.ae, examples/temporal_demo.ae.
Run all tests.


═══════════════════════════════════════════════════════════════
SESSION 9: New OOP Part 2 (Structural Mechanics)
═══════════════════════════════════════════════════════════════

Read the existing codebase.

Implement the structural OOP concepts:

1. weave: Parse weave definitions (before/after/around). When a class has "with Logged", inject the weave's before/after code into every method at AST level before interpretation.

2. bond: Parse bond declarations. When a bond property is modified (push/remove), auto-generate the reciprocal operation on the other side.

3. face: Parse face declarations inside classes. Implement .as(.face_name) that returns a restricted view object where only declared fields are accessible.

4. extend: Parse extend blocks. Register extension methods on types. When a method call is not found on the type, check extension methods.

5. delegate: Parse delegate declarations. For every method on the delegated type that isn't overridden, auto-forward the call to the inner object.

6. select/exclude: When parsing "class X : Parent select(a, b, c)", only inherit the listed methods. When parsing "exclude(a, b)", inherit everything except listed.

7. morph: Parse morph methods with when conditions. At compile time (or interpretation time), evaluate the condition and select the matching implementation.

8. capabilities: Parse "with capabilities(read_file, network)". When the class body calls a function that requires a capability not listed, report an error.

Create examples/weave_demo.ae, examples/bond_demo.ae, examples/face_demo.ae, examples/selective_demo.ae.
Run all tests.


═══════════════════════════════════════════════════════════════
SESSION 10: Genetic Classes
═══════════════════════════════════════════════════════════════

Read the existing codebase.

Implement genetic classes:

1. Parse genetic class declarations with chromosome { } blocks containing gene declarations
2. Each gene has: name, type, default value, constraints (range, options, step, scale, when condition)
3. Store gene values in a GeneticInstance struct
4. Implement .mutate() operations:
   - .mutate(.point) — change one random gene within its valid range
   - .mutate(.chromosome, target: name) — randomize all genes in one chromosome
   - .mutate(.insert, gene: def) — add a new gene
   - .mutate(.delete, gene: name) — revert gene to default
   - .mutate(.invert, genes: [a, b]) — swap two gene values
   - Custom: .mutate { genes in ... }
5. Implement crossover(a, b) — create child by taking chromosomes from each parent
6. Implement breed(a, b, mutation_rate: f) — crossover then mutate
7. Implement evolve Block { population: N, generations: N, mutation_rate: f, crossover_rate: f, selection: method, elitism: f, fitness function }
   - Create initial population of N random variants
   - Each generation: evaluate fitness, select parents, crossover, mutate, replace
   - Selection methods: .tournament(size: N), .roulette, .rank
   - Elitism: top X% survive unchanged
   - Return the fittest individual
8. .genes() returns current gene values as a map
9. .last_fitness returns the last evaluated fitness score

Create examples/genetic_demo.ae:
```
genetic class Strategy {
    chromosome params {
        gene threshold: Float = 0.5 { range 0.0..1.0 }
        gene window: Int = 20 { range 5..200 }
    }
    fitness(data: Float[]) -> Float {
        correct = 0
        for val in data {
            if val > self.threshold { correct += 1 }
        }
        return correct as Float / data.len() as Float
    }
}

data = [0.1, 0.3, 0.5, 0.7, 0.9, 0.2, 0.8, 0.4, 0.6, 0.95]
best = evolve Strategy {
    population: 50
    generations: 20
    fitness on data: data
}
print("Best threshold: {best.threshold}")
print("Best fitness: {best.last_fitness}")
```

Run it. Verify evolution actually improves fitness over generations.


═══════════════════════════════════════════════════════════════
SESSION 11: Bytecode Compiler + VM
═══════════════════════════════════════════════════════════════

Read the existing codebase.

Implement a bytecode compiler and stack-based VM for faster execution:

1. Define bytecode instruction set in src/compiler/bytecode.rs:
   - Stack ops: Push, Pop, Dup
   - Arithmetic: Add, Sub, Mul, Div, Mod, Pow, Neg
   - Comparison: Eq, Ne, Lt, Gt, Le, Ge
   - Logic: And, Or, Not
   - Variables: LoadLocal, StoreLocal, LoadGlobal, StoreGlobal
   - Functions: Call, Return
   - Jumps: Jump, JumpIfFalse, JumpIfTrue, Loop
   - Objects: GetField, SetField, CreateInstance
   - Collections: CreateList, CreateMap, Index, SetIndex
   - Constants: LoadConst (index into constant pool)
   - String interpolation: BuildString(part_count)

2. Implement compiler in src/compiler/compiler.rs:
   - Walk the AST and emit bytecode
   - Constant pool for literals
   - Local variable slots
   - Function compilation (separate chunk per function)

3. Implement VM in src/compiler/vm.rs:
   - Stack-based execution
   - Call stack with frames
   - Garbage collection (simple mark-and-sweep or just use Rc)

4. Add CLI option: aether run file.ae (uses interpreter by default), aether run --vm file.ae (uses bytecode VM)

5. Benchmark: run the same program on interpreter vs VM, print timing comparison.

Run all tests with both interpreter and VM modes.


═══════════════════════════════════════════════════════════════
SESSION 12: Cranelift Native Code Generation
═══════════════════════════════════════════════════════════════

Read the existing codebase.

Implement native code generation using Cranelift:

1. src/codegen/cranelift.rs:
   - Convert bytecode (or AST directly) to Cranelift IR
   - Function compilation: parameters, locals, return values
   - Arithmetic operations -> Cranelift iadd, isub, imul, etc.
   - Comparisons -> Cranelift icmp
   - Control flow -> Cranelift blocks with jumps
   - Function calls -> Cranelift call instruction
   - String handling via runtime helper functions

2. Use cranelift-jit for JIT compilation (compile and run in memory)
3. Use cranelift-module + cranelift-object for AOT (produce .o files, link to executable)

4. CLI: aether build file.ae — produces standalone native binary
   aether build file.ae -o myapp — custom output name

Start with compiling simple programs (arithmetic, functions, if/else).
Incrementally add support for more features.
Not everything needs to compile to native — fall back to interpreter for unsupported features.

Test: cargo run -- build examples/hello.ae && ./hello


═══════════════════════════════════════════════════════════════
SESSION 13: Tensor Library + GPU CUDA PTX
═══════════════════════════════════════════════════════════════

Read the existing codebase.

1. Implement Tensor<Float32> type in src/stdlib/tensor.rs:
   - Creation: Tensor.zeros(shape), Tensor.ones(shape), Tensor.random(shape), Tensor.from_list(data)
   - Shape: .shape, .dim, .len()
   - Element-wise: +, -, *, / (tensor op tensor, tensor op scalar)
   - Matrix multiply: ** operator
   - Reduction: .sum(), .mean(), .max(), .min(), .std()
   - Slicing: tensor[0..10, :, 3..]
   - Transpose: .T
   - Reshape: .reshape(new_shape)
   - Display: print(tensor) shows formatted output

2. Implement @gpu -> CUDA PTX generation in src/codegen/cuda.rs:
   - When a function has @gpu decorator, compile its body to PTX assembly text
   - Start with element-wise operations: add, sub, mul, div on tensors
   - Then matrix multiply (tiled matmul kernel)
   - Generate valid PTX with proper headers, register declarations, kernel entry
   - Test: write PTX to .ptx file, compile with nvcc -ptx, run on GH200

3. Basic autodiff in src/stdlib/tensor.rs:
   - gradient(loss, for: params) — reverse-mode automatic differentiation
   - Track computation graph during forward pass
   - Backpropagate gradients

Create examples/tensor_demo.ae and examples/gpu_demo.ae.
Test on GH200: verify CUDA kernels actually execute and produce correct results.


═══════════════════════════════════════════════════════════════
SESSION 14: WebGPU + ROCm + Python Bridge
═══════════════════════════════════════════════════════════════

Read the existing codebase.

1. WebGPU WGSL generation in src/codegen/wgsl.rs:
   - Generate WGSL compute shader code from @gpu functions
   - Same tensor operations as CUDA but in WGSL syntax
   - Generate HTML wrapper that loads and runs the shader (for browser demos)

2. ROCm HIP generation in src/codegen/hip.rs:
   - Generate HIP source code (nearly identical to CUDA syntax)
   - HIP is source-compatible with CUDA — translate PTX logic to HIP

3. Python bridge in src/bridge/python.rs:
   - Use pyo3 to embed CPython
   - "use python.torch as torch" — import Python modules into Aether
   - Call Python functions from Aether
   - Pass Aether values to Python and back
   - Tensor interop via DLPack (zero-copy sharing)

Add pyo3 to Cargo.toml dependencies (uncomment it).
Create examples/python_bridge.ae:
```
use python.numpy as np
arr = np.array([1, 2, 3, 4, 5])
print("Sum: {np.sum(arr)}")
```

Test Python bridge. Test WebGPU WGSL generation (output the .wgsl file).


═══════════════════════════════════════════════════════════════
SESSION 15: Forge Package Manager + aether convert
═══════════════════════════════════════════════════════════════

Read the existing codebase.

1. Forge package manager in src/forge/:
   - aether new <name> — create project with aether.toml, src/main.ae, tests/
   - aether build — compile project
   - aether run — build and run
   - aether test — run test files
   - aether.toml parser: [project] name, version, [dependencies] name = "version"
   - Dependency resolution (simple: no SAT solver needed for v1)
   - aether lint — format code (basic pretty-printer)

2. aether convert (Python transpiler):
   - Parse Python source using a simple Python parser (or regex-based for common patterns)
   - Convert Python constructs to Aether equivalents:
     - def f(x): -> def f(x) { }
     - if x: -> if x { }
     - for x in y: -> for x in y { }
     - self.x -> self.x
     - __init__ -> init
     - import x -> use x
     - print(f"...") -> print("...")
   - Output .ae file
   - CLI: aether convert input.py --output output.ae

Test with a simple Python file converted to Aether.


═══════════════════════════════════════════════════════════════
SESSION 16: VS Code Extension
═══════════════════════════════════════════════════════════════

Read the existing codebase.

Create the VS Code extension in aether-vscode/ directory:

1. package.json with extension manifest, language configuration
2. syntaxes/aether.tmLanguage.json — TextMate grammar for .ae files:
   - All keywords highlighted
   - String interpolation {expr} highlighted differently
   - Comments, decorators, types, numbers
   - Match arms, parallel blocks
3. language-configuration.json — bracket matching, auto-close, comment toggling
4. Basic Language Server Protocol (LSP):
   - Start the Aether compiler in LSP mode
   - Report diagnostics (type errors, parse errors) as red squiggles
   - Hover: show type of variable/function
   - Go to definition: jump to function/class source
   - Autocomplete: suggest keywords, local variables, methods
5. Snippets: for, loop, class, struct, enum, parallel, match, def, if
6. Custom theme: aether-dark with navy/purple/green colors

Build: cd aether-vscode && npm install && npm run compile
Package: npx vsce package (produces .vsix file)


═══════════════════════════════════════════════════════════════
SESSION 17: AetherStudio IDE (VSCodium Fork)
═══════════════════════════════════════════════════════════════

Clone VSCodium and customize it into AetherStudio:

1. git clone https://github.com/VSCodium/vscodium.git aether-studio
2. Rebrand: Change product name, icons, splash screen in product.json
3. Pre-bundle the Aether VS Code extension from Session 16
4. Add custom welcome tab with Aether getting-started tutorial
5. Add custom panels (VS Code webview extensions):
   - Tensor Inspector: visualize tensor shapes and values
   - GPU Profiler: show GPU utilization when running @gpu code
   - REPL panel: integrated Aether REPL in the terminal
6. Custom default theme: Aether Dark
7. Remove unnecessary default extensions
8. Build: follow VSCodium build instructions for your platform

This session may not complete fully — the build system is complex. Get as far as possible. The VS Code extension alone (from Session 16) provides 90% of the IDE value.


═══════════════════════════════════════════════════════════════
SESSION 18: Demo Programs + Polish + README
═══════════════════════════════════════════════════════════════

Read the ENTIRE codebase. Fix ALL remaining bugs, warnings, and clippy lints.

1. Write 10 polished demo programs in examples/:
   - hello.ae: Hello world + basic variables + string interpolation
   - parallel_timing.ae: Show 3x speedup with parallel {} vs sequential
   - pattern_matching.ae: Complex enum + match with destructuring
   - classes_oop.ae: Classes, inheritance, interfaces, operator overloading
   - reactive_demo.ae: Shopping cart with reactive total/count
   - temporal_demo.ae: Sensor with history tracking and trend detection
   - mutation_demo.ae: Bank account with tracked/undoable balance + atomic transfer
   - genetic_evolution.ae: Evolve a strategy over generations
   - type_safety.ae: Show compile-time error catching in #strict mode
   - full_stack.ae: Complete program using every major feature

2. Write README.md for GitHub:
   - Logo/title
   - One-paragraph description
   - Quick install instructions
   - "Hello World" example
   - 5 feature highlights with code snippets
   - Link to documentation
   - Contributing guide
   - License (Apache 2.0)

3. Create LICENSE file (Apache 2.0 full text)
4. Create CONTRIBUTING.md
5. Create .github/workflows/ci.yml for GitHub Actions CI

Run EVERY demo program. Verify output. Fix any remaining bugs.
Run cargo test with --release. All tests must pass.
Run cargo clippy -- -W warnings. Fix all warnings.


═══════════════════════════════════════════════════════════════
SESSION 19: Website + Online Playground
═══════════════════════════════════════════════════════════════

Create the Aether website in a website/ directory:

1. index.html — Landing page (dark theme, hero with code example, feature cards)
2. docs.html — Documentation with sidebar (all syntax, all features)
3. install.html — Installation for macOS/Linux/Windows
4. community.html — GitHub, Discord, contributing, roadmap
5. playground.html — In-browser code editor with example programs
6. style.css — Shared dark theme

If possible, compile the Aether interpreter to WASM so the playground actually runs Aether code in the browser. If not possible in this session, use a simulated output approach.


═══════════════════════════════════════════════════════════════
SESSION 20: Final Integration Test + Launch Prep
═══════════════════════════════════════════════════════════════

Final session. Read the entire codebase.

1. Run every example program. Fix any failures.
2. Run the complete test suite. Fix any failures.
3. Profile performance: interpreter vs VM vs native (Cranelift)
4. Test GPU on GH200: run tensor operations, verify CUDA PTX executes
5. Test Python bridge: import numpy, run a computation
6. Test parallel blocks: verify timing improvement
7. Test genetic classes: verify evolution improves fitness
8. Generate final build: cargo build --release
9. Test the install script
10. Verify VS Code extension installs and highlights .ae files
11. Create a 2-minute demo script (what to type/show for investors)

Commit everything to git. Tag as v0.1.0-alpha.
Print summary: total lines of code, number of tests, features implemented.
