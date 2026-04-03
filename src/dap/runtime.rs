use std::collections::HashMap;
use crate::interpreter::environment::Environment;
use crate::interpreter::values::Value;

/// Debug runtime state — wraps the interpreter with breakpoints and stepping.
pub struct DebugRuntime {
    pub env: Environment,
    pub breakpoints: HashMap<String, Vec<u32>>, // file -> line numbers
    pub paused: bool,
    pub current_file: String,
    pub current_line: u32,
    pub step_mode: StepMode,
    pub call_stack: Vec<StackFrame>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepMode {
    Continue,
    StepOver,
    StepInto,
    StepOut,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub name: String,
    pub file: String,
    pub line: u32,
    pub variables: Vec<(String, String)>, // (name, value_string)
}

impl DebugRuntime {
    pub fn new() -> Self {
        let mut env = Environment::new();
        crate::interpreter::register_builtins(&mut env);
        Self {
            env,
            breakpoints: HashMap::new(),
            paused: false,
            current_file: String::new(),
            current_line: 0,
            step_mode: StepMode::Continue,
            call_stack: Vec::new(),
        }
    }

    pub fn set_breakpoint(&mut self, file: &str, line: u32) {
        self.breakpoints.entry(file.to_string()).or_default().push(line);
    }

    pub fn remove_breakpoint(&mut self, file: &str, line: u32) {
        if let Some(lines) = self.breakpoints.get_mut(file) {
            lines.retain(|l| *l != line);
        }
    }

    pub fn should_pause(&self, file: &str, line: u32) -> bool {
        match self.step_mode {
            StepMode::StepOver | StepMode::StepInto => true,
            StepMode::Continue => {
                if let Some(lines) = self.breakpoints.get(file) {
                    lines.contains(&line)
                } else {
                    false
                }
            }
            StepMode::StepOut => false,
        }
    }

    pub fn get_variables(&self) -> Vec<(String, String)> {
        let mut vars = Vec::new();
        // Get all visible variables from the environment
        for val in self.env.all_values() {
            // We'd need to also get names — simplified for now
        }
        vars
    }

    /// Run a file with debug support.
    pub fn run_file(&mut self, source: &str, filename: &str) -> Result<(), String> {
        self.current_file = filename.to_string();

        let mut scanner = crate::lexer::scanner::Scanner::new(source, filename.to_string());
        let tokens = scanner.scan_tokens();
        let mut parser = crate::parser::parser::Parser::new(tokens);

        let program = parser.parse_program().map_err(|errors| {
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
        })?;

        // Execute with debug hooks
        crate::interpreter::interpret(&program, &mut self.env)
    }
}
