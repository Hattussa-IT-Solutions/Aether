# Aether Language for Visual Studio Code

Full IDE support for the **Aether programming language** — IntelliSense, debugging, syntax highlighting, and more.

## Features

### IntelliSense & Autocomplete
- Keyword, type, and function completions
- Method completions after `.` (knows String, List, Map, Set methods)
- 11 smart code snippets (class, function, match, parallel, genetic, etc.)
- Parameter hints and documentation on hover

### Live Diagnostics
- Parse errors shown as red squiggles in real-time
- Type checking with `#strict` mode support
- Error messages with file, line, and column

### Go to Definition
- Click any function, class, variable, or type to jump to its definition
- Works for `def`, `class`, `struct`, `enum`, and variable declarations

### Debugging
- Set breakpoints by clicking the gutter
- Step over, step into, step out
- View call stack and variables
- Evaluate expressions in the debug console
- Press **F5** to start debugging

### Run & REPL
- **Ctrl+Shift+R** (Cmd+Shift+R on Mac) to run the current file
- Integrated REPL terminal
- Run button in the editor title bar

### Syntax Highlighting
- All Aether keywords, operators, and types
- String interpolation `{expr}` highlighted
- Decorators `@gpu`, `@test`
- Directives `#strict`, `#test`
- Comments, numbers, characters

## Requirements

Install the Aether compiler:

```bash
curl -sSf https://raw.githubusercontent.com/Hattussa-IT-Solutions/Aether/main/install.sh | sh
```

Or build from source:

```bash
git clone https://github.com/Hattussa-IT-Solutions/Aether.git
cd Aether && cargo build --release
```

## Extension Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `aether.path` | `aether` | Path to the aether binary |
| `aether.checkOnSave` | `true` | Run type checker on save |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+R` / `Cmd+Shift+R` | Run current file |
| `F5` | Start debugging |

## Quick Start

1. Install this extension
2. Install Aether (`curl -sSf https://raw.githubusercontent.com/Hattussa-IT-Solutions/Aether/main/install.sh | sh`)
3. Create a file `hello.ae`:
   ```
   print("Hello from Aether!")
   name = "World"
   print("Hello, {name}!")
   ```
4. Press **Ctrl+Shift+R** to run it

## Links

- [GitHub](https://github.com/Hattussa-IT-Solutions/Aether)
- [Language Documentation](https://github.com/Hattussa-IT-Solutions/Aether#-language-tour)
- [Report Issues](https://github.com/Hattussa-IT-Solutions/Aether/issues)
