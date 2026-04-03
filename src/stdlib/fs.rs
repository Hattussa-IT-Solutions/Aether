use std::rc::Rc;
use std::cell::RefCell;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // ── existing functions kept for compatibility ──────────────────────────

    // fs_read(path) -> Str
    env.define("fs_read", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_read".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_read")?;
            std::fs::read_to_string(&path).map(Value::String).map_err(|e| e.to_string())
        }),
    })));

    // fs_write(path, content)
    env.define("fs_write", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_write".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_write")?;
            let content = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => return Err("fs_write requires 2 args".into()),
            };
            std::fs::write(&path, &content).map(|_| Value::Nil).map_err(|e| e.to_string())
        }),
    })));

    // fs_exists(path) -> Bool
    env.define("fs_exists", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_exists".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_exists")?;
            Ok(Value::Bool(std::path::Path::new(&path).exists()))
        }),
    })));

    // fs_lines(path) -> List<Str>
    env.define("fs_lines", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_lines".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_lines")?;
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let lines: Vec<Value> = content.lines().map(|l| Value::String(l.to_string())).collect();
            Ok(Value::List(Rc::new(RefCell::new(lines))))
        }),
    })));

    // ── new functions ──────────────────────────────────────────────────────

    // fs_append(path, content)
    env.define("fs_append", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_append".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_append")?;
            let content = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => return Err("fs_append requires 2 args".into()),
            };
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true).create(true).open(&path)
                .map_err(|e| e.to_string())?;
            file.write_all(content.as_bytes()).map(|_| Value::Nil).map_err(|e| e.to_string())
        }),
    })));

    // fs_delete(path)
    env.define("fs_delete", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_delete".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_delete")?;
            let p = std::path::Path::new(&path);
            if p.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            }
            .map(|_| Value::Nil)
            .map_err(|e| e.to_string())
        }),
    })));

    // fs_copy(src, dst)
    env.define("fs_copy", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_copy".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let src = str_arg(&args, 0, "fs_copy")?;
            let dst = str_arg(&args, 1, "fs_copy")?;
            std::fs::copy(&src, &dst).map(|_| Value::Nil).map_err(|e| e.to_string())
        }),
    })));

    // fs_rename(src, dst)
    env.define("fs_rename", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_rename".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let src = str_arg(&args, 0, "fs_rename")?;
            let dst = str_arg(&args, 1, "fs_rename")?;
            std::fs::rename(&src, &dst).map(|_| Value::Nil).map_err(|e| e.to_string())
        }),
    })));

    // fs_mkdir(path)  — one level only
    env.define("fs_mkdir", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_mkdir".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_mkdir")?;
            std::fs::create_dir(&path).map(|_| Value::Nil).map_err(|e| e.to_string())
        }),
    })));

    // fs_mkdir_all(path)  — create all intermediate dirs
    env.define("fs_mkdir_all", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_mkdir_all".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_mkdir_all")?;
            std::fs::create_dir_all(&path).map(|_| Value::Nil).map_err(|e| e.to_string())
        }),
    })));

    // fs_size(path) -> Int (bytes)
    env.define("fs_size", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_size".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_size")?;
            let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
            Ok(Value::Int(meta.len() as i64))
        }),
    })));

    // fs_is_file(path) -> Bool
    env.define("fs_is_file", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_is_file".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_is_file")?;
            Ok(Value::Bool(std::path::Path::new(&path).is_file()))
        }),
    })));

    // fs_is_dir(path) -> Bool
    env.define("fs_is_dir", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_is_dir".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_is_dir")?;
            Ok(Value::Bool(std::path::Path::new(&path).is_dir()))
        }),
    })));

    // fs_list_dir(path) -> List<Str>
    env.define("fs_list_dir", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_list_dir".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_list_dir")?;
            let entries = std::fs::read_dir(&path).map_err(|e| e.to_string())?;
            let mut names: Vec<Value> = Vec::new();
            for entry in entries {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry.file_name().to_string_lossy().into_owned();
                names.push(Value::String(name));
            }
            names.sort_by(|a, b| {
                if let (Value::String(sa), Value::String(sb)) = (a, b) {
                    sa.cmp(sb)
                } else {
                    std::cmp::Ordering::Equal
                }
            });
            Ok(Value::List(Rc::new(RefCell::new(names))))
        }),
    })));

    // fs_glob(pattern) -> List<Str>
    env.define("fs_glob", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_glob".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let pattern = str_arg(&args, 0, "fs_glob")?;
            let mut results: Vec<Value> = Vec::new();
            match glob::glob(&pattern) {
                Ok(paths) => {
                    for entry in paths.flatten() {
                        results.push(Value::String(entry.to_string_lossy().into_owned()));
                    }
                }
                Err(e) => return Err(format!("fs_glob: {}", e)),
            }
            Ok(Value::List(Rc::new(RefCell::new(results))))
        }),
    })));

    // fs_ext(path) -> Str  (file extension without dot, or "" if none)
    env.define("fs_ext", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_ext".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_ext")?;
            let ext = std::path::Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            Ok(Value::String(ext))
        }),
    })));

    // fs_stem(path) -> Str  (filename without extension)
    env.define("fs_stem", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_stem".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_stem")?;
            let stem = std::path::Path::new(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            Ok(Value::String(stem))
        }),
    })));

    // fs_parent(path) -> Str
    env.define("fs_parent", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_parent".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_parent")?;
            let parent = std::path::Path::new(&path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            Ok(Value::String(parent))
        }),
    })));

    // fs_join(parts) -> Str  — joins a list of path components
    env.define("fs_join", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_join".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let parts = match args.first() {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("fs_join: argument must be a list of strings".into()),
            };
            if parts.is_empty() {
                return Ok(Value::String(String::new()));
            }
            let mut path = std::path::PathBuf::new();
            for part in &parts {
                match part {
                    Value::String(s) => path.push(s),
                    _ => return Err("fs_join: all list elements must be strings".into()),
                }
            }
            Ok(Value::String(path.to_string_lossy().into_owned()))
        }),
    })));

    // fs_absolute(path) -> Str
    env.define("fs_absolute", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_absolute".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = str_arg(&args, 0, "fs_absolute")?;
            std::fs::canonicalize(&path)
                .map(|p| Value::String(p.to_string_lossy().into_owned()))
                .map_err(|e| e.to_string())
        }),
    })));

    // fs_temp_dir() -> Str
    env.define("fs_temp_dir", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_temp_dir".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::String(std::env::temp_dir().to_string_lossy().into_owned()))
        }),
    })));

    // fs_cwd() -> Str
    env.define("fs_cwd", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_cwd".into(),
        arity: Some(0),
        func: Box::new(|_| {
            std::env::current_dir()
                .map(|p| Value::String(p.to_string_lossy().into_owned()))
                .map_err(|e| e.to_string())
        }),
    })));
}

/// Helper: extract a string argument at position `idx`.
fn str_arg(args: &[Value], idx: usize, fn_name: &str) -> Result<String, String> {
    match args.get(idx) {
        Some(Value::String(s)) => Ok(s.clone()),
        _ => Err(format!("{}: argument {} must be a string", fn_name, idx + 1)),
    }
}
