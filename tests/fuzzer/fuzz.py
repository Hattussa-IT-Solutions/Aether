#!/usr/bin/env python3
"""Generate random valid Aether programs and run them to find crashes."""
import random
import subprocess
import os
import sys

def random_expr(depth=0):
    if depth > 3:
        return random.choice(["42", "3.14", '"hello"', "true", "false", "nil"])
    choices = [
        lambda: str(random.randint(-100, 100)),
        lambda: f"{random.uniform(-10, 10):.2f}",
        lambda: f'"{"".join(random.choice("abcdefghij ") for _ in range(random.randint(0,10)))}"',
        lambda: random.choice(["true", "false", "nil"]),
        lambda: f"({random_expr(depth+1)} {random.choice(['+','-','*'])} {random_expr(depth+1)})",
        lambda: f"({random_expr(depth+1)} {random.choice(['==','!=','<','>'])} {random_expr(depth+1)})",
        lambda: f"[{', '.join(random_expr(depth+1) for _ in range(random.randint(0,3)))}]",
    ]
    return random.choice(choices)()

def random_stmt(depth=0):
    if depth > 2:
        return f"x_{random.randint(0,20)} = {random_expr()}"
    choices = [
        lambda: f"x_{random.randint(0,20)} = {random_expr()}",
        lambda: f"print({random_expr()})",
        lambda: f"if {random_expr()} {{\n  x_{random.randint(0,20)} = {random_expr()}\n}}",
        lambda: f"for i in 0..{random.randint(1,5)} {{\n  x_{random.randint(0,20)} = {random_expr()}\n}}",
    ]
    return random.choice(choices)()

def random_program():
    return '\n'.join(random_stmt() for _ in range(random.randint(1, 15)))

def run_test(program, aether_binary, timeout=5):
    with open('/tmp/fuzz_test.ae', 'w') as f:
        f.write(program)
    try:
        result = subprocess.run(
            [aether_binary, 'run', '/tmp/fuzz_test.ae'],
            capture_output=True, text=True, timeout=timeout
        )
        if result.returncode < 0:
            return 'CRASH', result.stderr
        return 'OK', None
    except subprocess.TimeoutExpired:
        return 'TIMEOUT', None
    except Exception as e:
        return 'ERROR', str(e)

if __name__ == '__main__':
    aether = sys.argv[1] if len(sys.argv) > 1 else '../../target/release/aether'
    count = int(sys.argv[2]) if len(sys.argv) > 2 else 10000
    crashes = 0; timeouts = 0; ok = 0; errors = 0
    os.makedirs('crashes', exist_ok=True)

    for i in range(count):
        prog = random_program()
        status, err = run_test(prog, aether)
        if status == 'CRASH':
            crashes += 1
            with open(f'crashes/crash_{crashes}.ae', 'w') as f:
                f.write(prog)
            print(f"CRASH #{crashes}: {err[:100]}")
        elif status == 'TIMEOUT':
            timeouts += 1
        elif status == 'OK':
            ok += 1
        else:
            errors += 1
        if (i + 1) % 1000 == 0:
            print(f"Progress: {i+1}/{count} (OK:{ok} Crash:{crashes} Timeout:{timeouts} Error:{errors})")

    print(f"\n{'='*40}")
    print(f"Total: {count}")
    print(f"OK: {ok}  Crash: {crashes}  Timeout: {timeouts}  Error: {errors}")
    if crashes == 0:
        print("NO CRASHES FOUND!")
