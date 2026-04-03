use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {

    // ── memory_used() -> Int ─────────────────────────────────────
    // Reads /proc/self/statm on Linux; field 1 is RSS in pages.
    env.define("memory_used", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "memory_used".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::Int(read_memory_used()))
        }),
    })));

    // ── memory_peak() -> Int ─────────────────────────────────────
    // Reads VmPeak from /proc/self/status.
    env.define("memory_peak", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "memory_peak".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::Int(read_memory_peak()))
        }),
    })));

    // ── memory_gc() ──────────────────────────────────────────────
    // No-op hint: Rust uses RAII. We do nothing here because memory
    // is freed when Values drop out of scope.
    env.define("memory_gc", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "memory_gc".into(),
        arity: Some(0),
        func: Box::new(|_| {
            // Rust is RAII — no GC needed. This is a semantic no-op.
            Ok(Value::Nil)
        }),
    })));
}

/// Read current memory usage in bytes from /proc/self/status.
/// Tries VmRSS (physical), then VmSize (virtual), then VmPeak as last resort.
fn read_memory_used() -> i64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
            let mut vmrss: i64 = 0;
            let mut vmsize: i64 = 0;
            let mut vmpeak: i64 = 0;
            for line in contents.lines() {
                let parse_kb = |line: &str| -> i64 {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 { parts[1].parse::<i64>().unwrap_or(0) * 1024 }
                    else { 0 }
                };
                if line.starts_with("VmRSS:") { vmrss = parse_kb(line); }
                else if line.starts_with("VmSize:") { vmsize = parse_kb(line); }
                else if line.starts_with("VmPeak:") { vmpeak = parse_kb(line); }
            }
            if vmrss > 0 { return vmrss; }
            if vmsize > 0 { return vmsize; }
            if vmpeak > 0 { return vmpeak; }
        }
        // Fallback: read from /proc/self/statm (size field = virtual pages)
        if let Ok(contents) = std::fs::read_to_string("/proc/self/statm") {
            if let Some(size_pages) = contents.split_whitespace().next().and_then(|s| s.parse::<i64>().ok()) {
                if size_pages > 0 {
                    return size_pages * page_size();
                }
            }
        }
        0
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

/// Read VmPeak in bytes from /proc/self/status.
fn read_memory_peak() -> i64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
            for line in contents.lines() {
                if line.starts_with("VmPeak:") {
                    // Format: "VmPeak:   12345 kB"
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<i64>() {
                            return kb * 1024;
                        }
                    }
                }
            }
        }
        0
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

/// Get system page size.
fn page_size() -> i64 {
    #[cfg(target_os = "linux")]
    {
        // sysconf(_SC_PAGESIZE) — use a safe fallback of 4096 via libc call
        // We read it from /proc/self/smaps or just default to 4096.
        // Reading sysconf without libc: parse /proc/self/smaps KernelPageSize line.
        // Simplest safe approach: default 4096 (x86-64/arm64 standard).
        4096
    }
    #[cfg(not(target_os = "linux"))]
    {
        4096
    }
}
