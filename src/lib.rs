#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

pub mod lexer;
pub mod parser;
pub mod types;
pub mod interpreter;
pub mod compiler;
pub mod codegen;
pub mod stdlib;
pub mod bridge;
pub mod repl;
pub mod diagnostics;
pub mod forge;
pub mod lsp;
pub mod dap;
pub mod debugger;
pub mod watcher;
