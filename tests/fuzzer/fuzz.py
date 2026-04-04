#!/usr/bin/env python3
"""
Enhanced fuzzer for the Aether programming language.

Generates random programs across multiple categories and runs them
against the Aether binary to find crashes (signals/segfaults).

Categories:
  A - Deep nesting (nested ifs, loops, function chains, try/catch)
  B - Complex closures (lambdas, captured variables, higher-order functions)
  C - Method chains (list/string method chaining)
  D - Type mixing (mixed-type lists, type coercion edge cases)
  E - OOP edge cases (classes, inheritance, operator overloading)
  F - Resource stress (large strings, large lists, deep recursion)

Usage:
  python3 fuzz.py <aether_binary> <count>
  python3 fuzz.py ../../target/release/aether 10000
"""

import random
import subprocess
import os
import sys
import string
import time

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_var_counter = 0

def fresh_var(prefix="v"):
    global _var_counter
    _var_counter += 1
    return f"{prefix}_{_var_counter}"

def reset_var_counter():
    global _var_counter
    _var_counter = 0

def indent(code, level=1):
    pad = "    " * level
    return "\n".join(pad + line for line in code.split("\n") if line.strip())

def rand_ident():
    return random.choice(["x", "y", "z", "a", "b", "tmp", "val", "item", "n", "m"])

def rand_int():
    return str(random.randint(-500, 500))

def rand_float():
    return f"{random.uniform(-100, 100):.3f}"

def rand_str(max_len=15):
    length = random.randint(0, max_len)
    chars = string.ascii_lowercase + " "
    content = "".join(random.choice(chars) for _ in range(length))
    return f'"{content}"'

def rand_literal():
    return random.choice([
        rand_int, rand_float, rand_str,
        lambda: random.choice(["true", "false", "nil"]),
    ])()

def rand_expr(depth=0, max_depth=3):
    """Generate a random expression."""
    if depth >= max_depth:
        return rand_literal()
    builders = [
        lambda: rand_literal(),
        lambda: rand_ident(),
        # arithmetic
        lambda: f"({rand_expr(depth+1)} {random.choice(['+','-','*','/','%'])} {rand_expr(depth+1)})",
        # comparison
        lambda: f"({rand_expr(depth+1)} {random.choice(['==','!=','<','>','<=','>='])} {rand_expr(depth+1)})",
        # logical
        lambda: f"({rand_expr(depth+1)} {random.choice(['&&','||'])} {rand_expr(depth+1)})",
        # unary
        lambda: f"(!{rand_expr(depth+1)})",
        # list literal
        lambda: f"[{', '.join(rand_expr(depth+1) for _ in range(random.randint(0, 4)))}]",
        # map literal
        lambda: "{" + ", ".join(
            f'"{random.choice(string.ascii_lowercase)}": {rand_expr(depth+1)}'
            for _ in range(random.randint(0, 3))
        ) + "}",
        # string interpolation
        lambda: f'"val={{({rand_expr(depth+1)})}}"',
        # nil coalescing
        lambda: f"({rand_expr(depth+1)} ?? {rand_expr(depth+1)})",
        # power
        lambda: f"({rand_expr(depth+1)} ** {random.choice(['2', '3'])})",
    ]
    return random.choice(builders)()

def rand_simple_expr():
    """A simpler expression less likely to be deeply nested."""
    return rand_expr(depth=0, max_depth=1)

# ---------------------------------------------------------------------------
# Category A: Deep Nesting
# ---------------------------------------------------------------------------

def gen_nested_ifs(max_depth=10):
    """Generate deeply nested if statements."""
    depth = random.randint(3, max_depth)
    lines = []
    for i in range(depth):
        var = f"n_{i}"
        lines.append(f"{var} = {random.randint(0, 100)}")
    body = f'print("deep")'
    for i in range(depth - 1, -1, -1):
        var = f"n_{i}"
        op = random.choice([">", "<", "==", "!=", ">=", "<="])
        cmp_val = random.randint(0, 100)
        body = f"if {var} {op} {cmp_val} {{\n{indent(body)}\n}}"
        if random.random() < 0.3:
            body += f" else {{\n{indent('print(' + rand_str() + ')')}\n}}"
    return "\n".join(lines) + "\n" + body

def gen_nested_for_loops(max_depth=5):
    """Generate nested for loops up to max_depth."""
    depth = random.randint(2, max_depth)
    vars_used = [f"i_{j}" for j in range(depth)]
    acc = fresh_var("sum")
    inner = f"{acc} = {acc} + 1"
    for i in range(depth - 1, -1, -1):
        bound = random.randint(1, 4)
        inner = f"for {vars_used[i]} in 0..{bound} {{\n{indent(inner)}\n}}"
    return f"{acc} = 0\n{inner}\nprint({acc})"

def gen_function_chain():
    """Generate a chain of functions calling each other."""
    chain_len = random.randint(3, 8)
    funcs = [f"chain_{i}" for i in range(chain_len)]
    lines = []
    for i in range(chain_len):
        if i == chain_len - 1:
            lines.append(f"def {funcs[i]}(x) {{\n    return x + 1\n}}")
        else:
            op = random.choice(["+", "-", "*"])
            val = random.randint(1, 5)
            lines.append(f"def {funcs[i]}(x) {{\n    return {funcs[i+1]}(x {op} {val})\n}}")
    lines.append(f"print({funcs[0]}({random.randint(0, 10)}))")
    return "\n".join(lines)

def gen_nested_try_catch():
    """Generate nested try/catch blocks."""
    depth = random.randint(2, 5)
    inner = f'print("inner ok")'
    for _ in range(depth):
        risky = random.choice([
            f"{fresh_var()} = {rand_expr(max_depth=1)}",
            f'print({rand_expr(max_depth=1)})',
            f"{fresh_var()} = [1, 2, 3]\n    print({fresh_var()})",
        ])
        inner = (
            f"try {{\n"
            f"{indent(risky)}\n"
            f"{indent(inner)}\n"
            f"}} catch any as e {{\n"
            f'    print("caught")\n'
            f"}}"
        )
    return inner

def gen_category_a():
    return random.choice([
        gen_nested_ifs,
        gen_nested_for_loops,
        gen_function_chain,
        gen_nested_try_catch,
    ])()

# ---------------------------------------------------------------------------
# Category B: Complex Closures
# ---------------------------------------------------------------------------

def gen_lambda_capture_loop():
    """Lambda capturing loop variable."""
    lines = []
    list_var = fresh_var("fns")
    lines.append(f"{list_var} = []")
    lines.append(f"for i in 0..5 {{")
    lines.append(f"    {list_var}.push(i -> i * 2)")
    lines.append(f"}}")
    lines.append(f"for f in {list_var} {{")
    lines.append(f'    print("fn")')
    lines.append(f"}}")
    return "\n".join(lines)

def gen_closure_modify_outer():
    """Closure that modifies an outer variable."""
    outer = fresh_var("counter")
    lines = [
        f"{outer} = 0",
        f"def increment() {{",
        f"    {outer} = {outer} + 1",
        f"}}",
        f"increment()",
        f"increment()",
        f"increment()",
        f"print({outer})",
    ]
    return "\n".join(lines)

def gen_function_returning_function():
    """Function that returns another function."""
    fname = fresh_var("make_adder")
    lines = [
        f"def {fname}(n) {{",
        f"    return x -> x + n",
        f"}}",
        f"add5 = {fname}(5)",
        f"print(add5(10))",
    ]
    return "\n".join(lines)

def gen_nested_lambdas():
    """Multiple nested lambdas."""
    lines = [
        f"apply = (f, x) -> f(x)",
        f"double = x -> x * 2",
        f"result = apply(double, 21)",
        f"print(result)",
    ]
    return "\n".join(lines)

def gen_lambda_in_expression():
    """Lambda used inline in various contexts."""
    patterns = [
        'nums = [1, 2, 3, 4, 5]\nresult = nums.map(x -> x * 2)\nprint(result)',
        'nums = [10, 20, 30]\nresult = nums.filter(x -> x > 15)\nprint(result)',
        'items = [3, 1, 4, 1, 5]\nresult = items.filter(x -> x > 2).map(x -> x * 10)\nprint(result)',
    ]
    return random.choice(patterns)

def gen_category_b():
    return random.choice([
        gen_lambda_capture_loop,
        gen_closure_modify_outer,
        gen_function_returning_function,
        gen_nested_lambdas,
        gen_lambda_in_expression,
    ])()

# ---------------------------------------------------------------------------
# Category C: Method Chains
# ---------------------------------------------------------------------------

def gen_list_method_chain():
    """Chain of list operations."""
    list_init = f"[{', '.join(str(random.randint(-10, 10)) for _ in range(random.randint(3, 10)))}]"
    chain_ops = []
    for _ in range(random.randint(1, 4)):
        op = random.choice([
            ".map(x -> x * 2)",
            ".map(x -> x + 1)",
            ".filter(x -> x > 0)",
            ".filter(x -> x != 0)",
            ".len()",
            ".reverse()",
            ".contains(0)",
            ".first()",
            ".last()",
        ])
        chain_ops.append(op)
        # len/contains/first/last return non-list, so stop chaining
        if op in [".len()", ".contains(0)", ".first()", ".last()"]:
            break
    v = fresh_var("chain")
    return f"{v} = {list_init}{''.join(chain_ops)}\nprint({v})"

def gen_string_method_chain():
    """Chain of string operations."""
    s = rand_str(20)
    chain_ops = []
    for _ in range(random.randint(1, 4)):
        op = random.choice([
            ".upper()",
            ".lower()",
            ".trim()",
            ".len()",
            '.replace("a", "z")',
            '.split(" ")',
            '.contains("hello")',
            ".reverse()",
        ])
        chain_ops.append(op)
        # Some ops return non-string
        if op in [".len()", '.split(" ")', '.contains("hello")']:
            break
    v = fresh_var("s")
    return f'{v} = {s}{"".join(chain_ops)}\nprint({v})'

def gen_map_operations():
    """Map creation and operations."""
    v = fresh_var("m")
    lines = [
        f'{v} = {{"a": 1, "b": 2, "c": 3}}',
        f'print({v})',
        f'{v}["d"] = 4',
        f'print({v}.len())',
        f'print({v}.keys())',
        f'print({v}.values())',
    ]
    return "\n".join(lines)

def gen_category_c():
    return random.choice([
        gen_list_method_chain,
        gen_string_method_chain,
        gen_map_operations,
        gen_list_method_chain,  # weighted more
        gen_string_method_chain,
    ])()

# ---------------------------------------------------------------------------
# Category D: Type Mixing
# ---------------------------------------------------------------------------

def gen_mixed_type_list():
    """Lists with mixed types."""
    elements = [
        rand_int, lambda: rand_str(5), lambda: random.choice(["true", "false", "nil"]),
        rand_float, lambda: "[]", lambda: "[1, 2]",
    ]
    items = ", ".join(random.choice(elements)() for _ in range(random.randint(2, 6)))
    v = fresh_var("mixed")
    return f"{v} = [{items}]\nprint({v})\nprint({v}.len())"

def gen_type_coercion_ops():
    """Operations between different types that may produce errors."""
    ops = [
        '1 + "hello"',
        '"hello" + 1',
        'true + 1',
        '"abc" * 3',
        '"abc" - "a"',
        'nil + 1',
        '[1, 2] + [3, 4]',
        '[1, 2] + 3',
        'true && 1',
        '1 / 0',
        '0 / 0',
        '1 % 0',
        '1.0 / 0.0',
        '"hello" > 5',
        'nil == nil',
        'nil == false',
        'true == 1',
        '"" == false',
        '[] == false',
    ]
    selected = random.sample(ops, min(random.randint(3, 8), len(ops)))
    lines = []
    for op in selected:
        v = fresh_var("r")
        lines.append(f"try {{")
        lines.append(f"    {v} = {op}")
        lines.append(f"    print({v})")
        lines.append(f"}} catch any as e {{")
        lines.append(f'    print("error")')
        lines.append(f"}}")
    return "\n".join(lines)

def gen_nil_operations():
    """Various nil operations."""
    lines = [
        'a = nil',
        'b = a ?? "default"',
        'print(b)',
        'c = nil',
        'try {',
        '    d = c + 1',
        '    print(d)',
        '} catch any as e {',
        '    print("nil error caught")',
        '}',
        'print(nil == nil)',
        'print(nil != false)',
    ]
    return "\n".join(lines)

def gen_optional_chaining():
    """Test optional chaining and nil coalescing."""
    lines = [
        'x = nil',
        'y = x ?? 42',
        'print(y)',
        'z = nil',
        'w = z ?? "fallback"',
        'print(w)',
    ]
    return "\n".join(lines)

def gen_category_d():
    return random.choice([
        gen_mixed_type_list,
        gen_type_coercion_ops,
        gen_nil_operations,
        gen_optional_chaining,
    ])()

# ---------------------------------------------------------------------------
# Category E: OOP Edge Cases
# ---------------------------------------------------------------------------

def gen_class_many_fields():
    """Class with many fields and methods."""
    n_fields = random.randint(5, 15)
    cls_name = f"Big{fresh_var('Cls')}"
    field_names = [f"f{i}" for i in range(n_fields)]
    fields = "\n".join(f"    {f}: Int = {random.randint(0, 100)}" for f in field_names)
    methods = []
    for i in range(random.randint(1, 3)):
        mname = f"method_{i}"
        methods.append(f"    def {mname}() {{\n        return self.{random.choice(field_names)} + {random.randint(1, 10)}\n    }}")
    init_params = ", ".join(f"{f}: Int" for f in field_names[:3])
    init_body = "\n".join(f"        self.{f} = {f}" for f in field_names[:3])
    init_block = f"    init({init_params}) {{\n{init_body}\n    }}"
    class_body = f"class {cls_name} {{\n{fields}\n\n{init_block}\n\n{''.join(methods)}\n}}"
    # Instantiate
    args = ", ".join(str(random.randint(0, 50)) for _ in field_names[:3])
    return f"{class_body}\nobj = {cls_name}({args})\nprint(obj.{random.choice(field_names)})"

def gen_deep_inheritance():
    """Chain of classes inheriting from each other."""
    depth = random.randint(2, 5)
    names = [f"Level{i}_{fresh_var()}" for i in range(depth)]
    lines = []
    # Base class
    lines.append(f"class {names[0]} {{")
    lines.append(f"    val: Int = 0")
    lines.append(f"    def greet() {{ print(\"hello from {names[0]}\") }}")
    lines.append(f"}}")
    # Derived classes
    for i in range(1, depth):
        lines.append(f"class {names[i]} : {names[i-1]} {{")
        lines.append(f"    extra_{i}: Int = {i}")
        lines.append(f"    def level() {{ return {i} }}")
        lines.append(f"}}")
    lines.append(f"obj = {names[-1]}()")
    lines.append(f"obj.greet()")
    lines.append(f"print(obj.val)")
    return "\n".join(lines)

def gen_class_with_methods():
    """Class with various method types."""
    cls = fresh_var("MyClass")
    lines = [
        f"class {cls} {{",
        f"    x: Int = 0",
        f"    y: Int = 0",
        f"",
        f"    init(x: Int, y: Int) {{",
        f"        self.x = x",
        f"        self.y = y",
        f"    }}",
        f"",
        f"    def sum() {{",
        f"        return self.x + self.y",
        f"    }}",
        f"",
        f"    def scale(factor: Int) {{",
        f"        self.x = self.x * factor",
        f"        self.y = self.y * factor",
        f"    }}",
        f"",
        f"    def to_str() {{",
        f'        return "({{}}, {{}})".replace("{{}}", "{{}}")'.replace("{{}}", "{self.x}"),
        f"    }}",
        f"}}",
        f"",
        f"obj = {cls}({random.randint(1,10)}, {random.randint(1,10)})",
        f"print(obj.sum())",
        f"obj.scale(2)",
        f"print(obj.x)",
    ]
    return "\n".join(lines)

def gen_struct_usage():
    """Struct creation and field access."""
    name = fresh_var("Point")
    lines = [
        f"struct {name} {{",
        f"    x: Float = 0.0",
        f"    y: Float = 0.0",
        f"}}",
        f"p = {name}()",
        f"p.x = 3.0",
        f"p.y = 4.0",
        f"print(p.x)",
        f"print(p.y)",
    ]
    return "\n".join(lines)

def gen_class_self_reference():
    """Class methods that call other methods on self."""
    cls = fresh_var("Chain")
    lines = [
        f"class {cls} {{",
        f"    val: Int = 0",
        f"    def inc() {{",
        f"        self.val = self.val + 1",
        f"        return self",
        f"    }}",
        f"    def get() {{",
        f"        return self.val",
        f"    }}",
        f"}}",
        f"obj = {cls}()",
        f"obj.inc()",
        f"obj.inc()",
        f"obj.inc()",
        f"print(obj.get())",
    ]
    return "\n".join(lines)

def gen_category_e():
    return random.choice([
        gen_class_many_fields,
        gen_deep_inheritance,
        gen_class_with_methods,
        gen_struct_usage,
        gen_class_self_reference,
    ])()

# ---------------------------------------------------------------------------
# Category F: Resource Stress
# ---------------------------------------------------------------------------

def gen_large_string_concat():
    """Build a large string through concatenation."""
    v = fresh_var("big")
    n = random.randint(50, 500)
    lines = [
        f'{v} = ""',
        f"for i in 0..{n} {{",
        f'    {v} = {v} + "x"',
        f"}}",
        f"print({v}.len())",
    ]
    return "\n".join(lines)

def gen_large_list():
    """Create a large list."""
    v = fresh_var("big_list")
    n = random.randint(100, 1000)
    lines = [
        f"{v} = []",
        f"for i in 0..{n} {{",
        f"    {v}.push(i)",
        f"}}",
        f"print({v}.len())",
    ]
    return "\n".join(lines)

def gen_deep_recursion():
    """Recursive function that goes deep (within safety limits)."""
    fname = fresh_var("recurse")
    max_depth = random.randint(100, 900)
    lines = [
        f"def {fname}(n) {{",
        f"    if n <= 0 {{",
        f"        return 0",
        f"    }}",
        f"    return {fname}(n - 1) + 1",
        f"}}",
        f"print({fname}({max_depth}))",
    ]
    return "\n".join(lines)

def gen_many_variables():
    """Create many variables."""
    n = random.randint(50, 200)
    lines = [f"var_{i} = {random.randint(-1000, 1000)}" for i in range(n)]
    sum_expr = " + ".join(f"var_{i}" for i in range(min(n, 20)))
    lines.append(f"total = {sum_expr}")
    lines.append(f"print(total)")
    return "\n".join(lines)

def gen_nested_list_creation():
    """Create nested lists."""
    depth = random.randint(2, 6)
    inner = "[1, 2, 3]"
    for _ in range(depth):
        inner = f"[{inner}, {inner}]"
    return f"nested = {inner}\nprint(nested)"

def gen_string_operations_stress():
    """Many string operations."""
    v = fresh_var("s")
    lines = [f'{v} = "hello world this is a test string"']
    ops = [
        f"print({v}.upper())",
        f"print({v}.lower())",
        f"print({v}.len())",
        f'print({v}.split(" "))',
        f'print({v}.replace("hello", "goodbye"))',
        f"print({v}.trim())",
        f"print({v}.reverse())",
        f'print({v}.contains("world"))',
        f'print({v}.starts_with("hello"))',
        f'print({v}.ends_with("string"))',
    ]
    random.shuffle(ops)
    lines.extend(ops[:random.randint(3, len(ops))])
    return "\n".join(lines)

def gen_category_f():
    return random.choice([
        gen_large_string_concat,
        gen_large_list,
        gen_deep_recursion,
        gen_many_variables,
        gen_nested_list_creation,
        gen_string_operations_stress,
    ])()

# ---------------------------------------------------------------------------
# Mixed / Combo generators
# ---------------------------------------------------------------------------

def gen_match_statement():
    """Generate a match statement."""
    val_expr = random.choice([rand_int(), rand_str(5), "true", "false", "nil"])
    arms = []
    for _ in range(random.randint(2, 5)):
        pattern = random.choice([rand_int(), rand_str(5), "true", "false", "nil", "_"])
        action = random.choice([
            f'print({rand_str()})',
            f'{fresh_var()} = {rand_int()}',
        ])
        arms.append(f"    {pattern} -> {action}")
    if not any("_" in a for a in arms):
        arms.append(f'    _ -> print("default")')
    return f"match {val_expr} {{\n" + "\n".join(arms) + "\n}"

def gen_loop_variants():
    """Various loop forms."""
    variant = random.choice(["times", "while", "for_range", "for_list", "loop_break"])
    if variant == "times":
        n = random.randint(1, 10)
        return f'loop {n} times {{\n    print("tick")\n}}'
    elif variant == "while":
        v = fresh_var("cnt")
        return f"{v} = 0\nloop while {v} < {random.randint(1, 10)} {{\n    {v} = {v} + 1\n}}\nprint({v})"
    elif variant == "for_range":
        a, b = sorted([random.randint(0, 20), random.randint(0, 20)])
        return f"for i in {a}..{b} {{\n    print(i)\n}}"
    elif variant == "for_list":
        items = ", ".join(rand_literal() for _ in range(random.randint(1, 5)))
        return f"for item in [{items}] {{\n    print(item)\n}}"
    else:
        v = fresh_var("c")
        return f"{v} = 0\nloop {{\n    {v} = {v} + 1\n    if {v} >= 5 {{\n        break\n    }}\n}}\nprint({v})"

def gen_pipeline():
    """Pipeline operator usage."""
    lines = [
        "def double(x) { return x * 2 }",
        "def inc(x) { return x + 1 }",
        f"result = {random.randint(1, 10)} |> double |> inc",
        "print(result)",
    ]
    return "\n".join(lines)

def gen_comprehension():
    """List comprehension."""
    patterns = [
        f"squares = [x ** 2 for x in 0..{random.randint(3, 10)}]\nprint(squares)",
        f"evens = [x for x in 0..{random.randint(5, 20)} if x % 2 == 0]\nprint(evens)",
        f"pairs = [x * y for x in 0..{random.randint(2, 4)} for y in 0..{random.randint(2, 4)}]"
        f"\nprint(pairs)",
    ]
    return random.choice(patterns)

def gen_error_handling():
    """Various try/catch patterns."""
    patterns = [
        'try {\n    x = 1 / 0\n    print(x)\n} catch any as e {\n    print("caught division")\n}',
        'try {\n    x = nil\n    y = x + 1\n} catch any as e {\n    print("caught nil")\n}',
        'try {\n    items = [1, 2, 3]\n    print(items[100])\n} catch any as e {\n    print("caught index")\n}',
    ]
    return random.choice(patterns)

def gen_multiline_string_interp():
    """String interpolation edge cases."""
    v = fresh_var("name")
    lines = [
        f'{v} = "world"',
        f'greeting = "hello {{{v}}}"',
        f"print(greeting)",
        f'multi = "1+1 = {{(1 + 1)}}"',
        f"print(multi)",
    ]
    return "\n".join(lines)

def gen_next_break():
    """Test next (skip) and break in loops."""
    lines = [
        f"for i in 0..10 {{",
        f"    if i == 3 {{",
        f"        next",
        f"    }}",
        f"    if i == 7 {{",
        f"        break",
        f"    }}",
        f"    print(i)",
        f"}}",
    ]
    return "\n".join(lines)

def gen_combo():
    """Generate from the mixed/combo generators."""
    return random.choice([
        gen_match_statement,
        gen_loop_variants,
        gen_pipeline,
        gen_comprehension,
        gen_error_handling,
        gen_multiline_string_interp,
        gen_next_break,
    ])()

# ---------------------------------------------------------------------------
# Program Assembly
# ---------------------------------------------------------------------------

CATEGORIES = {
    "A": gen_category_a,
    "B": gen_category_b,
    "C": gen_category_c,
    "D": gen_category_d,
    "E": gen_category_e,
    "F": gen_category_f,
    "X": gen_combo,
}

def generate_program():
    """Generate one random Aether program."""
    reset_var_counter()
    # Pick 1-3 fragments and combine
    n_fragments = random.randint(1, 3)
    cats = random.choices(list(CATEGORIES.keys()), k=n_fragments)
    fragments = []
    for cat in cats:
        try:
            fragments.append(CATEGORIES[cat]())
        except Exception:
            fragments.append(f'print("fuzz")')
    # Optionally add a comment
    if random.random() < 0.2:
        fragments.insert(0, f"// fuzz test {random.randint(0, 99999)}")
    return "\n\n".join(fragments)

def generate_program_category(cat):
    """Generate a program from a specific category."""
    reset_var_counter()
    try:
        body = CATEGORIES[cat]()
    except Exception:
        body = f'print("fuzz")'
    if random.random() < 0.1:
        body = f"// fuzz category {cat}\n" + body
    return body

# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------

def run_test(program, aether_binary, timeout=10):
    """Run a single test program and return (status, stderr)."""
    tmp = "/tmp/aether_fuzz_test.ae"
    with open(tmp, "w") as f:
        f.write(program)
    try:
        result = subprocess.run(
            [aether_binary, "run", tmp],
            capture_output=True, text=True, timeout=timeout,
        )
        if result.returncode < 0:
            return "CRASH", result.stderr
        elif result.returncode > 0:
            return "ERROR", result.stderr
        else:
            return "OK", None
    except subprocess.TimeoutExpired:
        return "TIMEOUT", None
    except Exception as e:
        return "ERROR", str(e)

def main():
    aether = sys.argv[1] if len(sys.argv) > 1 else "../../target/release/aether"
    count = int(sys.argv[2]) if len(sys.argv) > 2 else 10000
    timeout = 10

    # Verify binary exists
    if not os.path.isfile(aether):
        # Try resolving relative to script dir
        script_dir = os.path.dirname(os.path.abspath(__file__))
        alt = os.path.join(script_dir, aether)
        if os.path.isfile(alt):
            aether = alt
        else:
            print(f"ERROR: Aether binary not found: {aether}")
            sys.exit(1)

    os.makedirs("crashes", exist_ok=True)
    os.makedirs("timeouts", exist_ok=True)

    stats = {"OK": 0, "ERROR": 0, "CRASH": 0, "TIMEOUT": 0}
    cat_stats = {c: {"OK": 0, "ERROR": 0, "CRASH": 0, "TIMEOUT": 0} for c in CATEGORIES}

    # Distribute roughly equally across categories
    cat_keys = list(CATEGORIES.keys())
    start_time = time.time()

    print(f"Aether Fuzzer - running {count} tests against {aether}")
    print(f"Categories: {', '.join(cat_keys)}")
    print(f"Timeout per program: {timeout}s")
    print("=" * 60)

    for i in range(count):
        cat = cat_keys[i % len(cat_keys)]
        prog = generate_program_category(cat)
        status, err = run_test(prog, aether, timeout=timeout)

        stats[status] += 1
        cat_stats[cat][status] += 1

        if status == "CRASH":
            crash_num = stats["CRASH"]
            path = f"crashes/crash_{crash_num}.ae"
            with open(path, "w") as f:
                f.write(prog)
            print(f"  *** CRASH #{crash_num} (cat {cat}): saved to {path}")
            if err:
                print(f"      stderr: {err[:200]}")

        elif status == "TIMEOUT":
            to_num = stats["TIMEOUT"]
            path = f"timeouts/timeout_{to_num}.ae"
            with open(path, "w") as f:
                f.write(prog)

        if (i + 1) % 1000 == 0:
            elapsed = time.time() - start_time
            rate = (i + 1) / elapsed if elapsed > 0 else 0
            print(
                f"  [{i+1}/{count}] "
                f"OK:{stats['OK']} ERROR:{stats['ERROR']} "
                f"CRASH:{stats['CRASH']} TIMEOUT:{stats['TIMEOUT']} "
                f"({rate:.1f} tests/sec)"
            )

    elapsed = time.time() - start_time

    # Final summary
    print("\n" + "=" * 60)
    print(f"FUZZER COMPLETE: {count} programs in {elapsed:.1f}s ({count/elapsed:.1f} tests/sec)")
    print("=" * 60)
    print(f"  OK:      {stats['OK']:>6}")
    print(f"  ERROR:   {stats['ERROR']:>6}  (graceful errors)")
    print(f"  CRASH:   {stats['CRASH']:>6}  (signals/segfaults)")
    print(f"  TIMEOUT: {stats['TIMEOUT']:>6}  (>{timeout}s)")
    print()
    print("Per-category breakdown:")
    print(f"  {'Cat':<4} {'OK':>6} {'ERROR':>6} {'CRASH':>6} {'TIMEOUT':>7}")
    print(f"  {'-'*4} {'-'*6} {'-'*6} {'-'*6} {'-'*7}")
    for cat in cat_keys:
        cs = cat_stats[cat]
        print(f"  {cat:<4} {cs['OK']:>6} {cs['ERROR']:>6} {cs['CRASH']:>6} {cs['TIMEOUT']:>7}")
    print()

    if stats["CRASH"] == 0:
        print("NO CRASHES FOUND -- all good!")
    else:
        print(f"FOUND {stats['CRASH']} CRASH(ES) -- see crashes/ directory")
        sys.exit(1)

if __name__ == "__main__":
    main()
