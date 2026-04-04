use std::io::{BufRead, BufReader, Read, Write};
use serde_json::{json, Value};
use crate::dap::runtime::{DebugRuntime, StepMode};

/// DAP (Debug Adapter Protocol) server for Aether debugging.
/// Communicates via stdin/stdout using the DAP JSON protocol.
pub fn run_dap() {
    eprintln!("Aether DAP server starting...");

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();
    let mut runtime = DebugRuntime::new();
    let mut seq = 1;

    while let Ok(msg) = read_dap_message(&mut reader) {

        let command = msg["command"].as_str().unwrap_or("");
        let request_seq = msg["seq"].as_i64().unwrap_or(0);

        match command {
            "initialize" => {
                let response = dap_response(request_seq, seq, "initialize", json!({
                    "supportsConfigurationDoneRequest": true,
                    "supportsFunctionBreakpoints": false,
                    "supportsConditionalBreakpoints": false,
                    "supportsEvaluateForHovers": true,
                    "supportsStepBack": false,
                }));
                send_dap_message(&mut writer, &response);
                seq += 1;

                // Send initialized event
                let event = json!({
                    "seq": seq,
                    "type": "event",
                    "event": "initialized"
                });
                send_dap_message(&mut writer, &event);
                seq += 1;
            }

            "launch" => {
                let args = &msg["arguments"];
                let program = args["program"].as_str().unwrap_or("");

                let response = dap_response(request_seq, seq, "launch", json!(null));
                send_dap_message(&mut writer, &response);
                seq += 1;

                // Run the program
                if !program.is_empty() {
                    match std::fs::read_to_string(program) {
                        Ok(source) => {
                            match runtime.run_file(&source, program) {
                                Ok(()) => {}
                                Err(e) => {
                                    let event = json!({
                                        "seq": seq,
                                        "type": "event",
                                        "event": "output",
                                        "body": {
                                            "category": "stderr",
                                            "output": format!("Error: {}\n", e)
                                        }
                                    });
                                    send_dap_message(&mut writer, &event);
                                    seq += 1;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Could not read file: {}", e);
                        }
                    }
                }

                // Send terminated event
                let event = json!({
                    "seq": seq,
                    "type": "event",
                    "event": "terminated"
                });
                send_dap_message(&mut writer, &event);
                seq += 1;
            }

            "setBreakpoints" => {
                let args = &msg["arguments"];
                let source_path = args["source"]["path"].as_str().unwrap_or("");
                let breakpoints = args["breakpoints"].as_array();

                let mut verified = Vec::new();
                if let Some(bps) = breakpoints {
                    for bp in bps {
                        let line = bp["line"].as_u64().unwrap_or(0) as u32;
                        runtime.set_breakpoint(source_path, line);
                        verified.push(json!({
                            "verified": true,
                            "line": line
                        }));
                    }
                }

                let response = dap_response(request_seq, seq, "setBreakpoints", json!({
                    "breakpoints": verified
                }));
                send_dap_message(&mut writer, &response);
                seq += 1;
            }

            "configurationDone" => {
                let response = dap_response(request_seq, seq, "configurationDone", json!(null));
                send_dap_message(&mut writer, &response);
                seq += 1;
            }

            "threads" => {
                let response = dap_response(request_seq, seq, "threads", json!({
                    "threads": [{ "id": 1, "name": "main" }]
                }));
                send_dap_message(&mut writer, &response);
                seq += 1;
            }

            "stackTrace" => {
                let frames: Vec<Value> = runtime.call_stack.iter().enumerate().map(|(i, f)| {
                    json!({
                        "id": i,
                        "name": f.name,
                        "source": { "path": f.file },
                        "line": f.line,
                        "column": 1
                    })
                }).collect();

                if frames.is_empty() {
                    let response = dap_response(request_seq, seq, "stackTrace", json!({
                        "stackFrames": [{
                            "id": 0,
                            "name": "<main>",
                            "source": { "path": runtime.current_file },
                            "line": runtime.current_line,
                            "column": 1
                        }],
                        "totalFrames": 1
                    }));
                    send_dap_message(&mut writer, &response);
                } else {
                    let response = dap_response(request_seq, seq, "stackTrace", json!({
                        "stackFrames": frames,
                        "totalFrames": frames.len()
                    }));
                    send_dap_message(&mut writer, &response);
                }
                seq += 1;
            }

            "continue" => {
                runtime.step_mode = StepMode::Continue;
                let response = dap_response(request_seq, seq, "continue", json!({ "allThreadsContinued": true }));
                send_dap_message(&mut writer, &response);
                seq += 1;
            }

            "next" => {
                runtime.step_mode = StepMode::StepOver;
                let response = dap_response(request_seq, seq, "next", json!(null));
                send_dap_message(&mut writer, &response);
                seq += 1;
            }

            "stepIn" => {
                runtime.step_mode = StepMode::StepInto;
                let response = dap_response(request_seq, seq, "stepIn", json!(null));
                send_dap_message(&mut writer, &response);
                seq += 1;
            }

            "evaluate" => {
                let args = &msg["arguments"];
                let expression = args["expression"].as_str().unwrap_or("");

                // Try to evaluate the expression in the current environment
                let result = eval_in_context(expression, &mut runtime);
                let response = dap_response(request_seq, seq, "evaluate", json!({
                    "result": result,
                    "variablesReference": 0
                }));
                send_dap_message(&mut writer, &response);
                seq += 1;
            }

            "disconnect" => {
                let response = dap_response(request_seq, seq, "disconnect", json!(null));
                send_dap_message(&mut writer, &response);
                break;
            }

            _ => {
                let response = dap_response(request_seq, seq, command, json!(null));
                send_dap_message(&mut writer, &response);
                seq += 1;
            }
        }
    }

    eprintln!("Aether DAP server stopped");
}

fn eval_in_context(expr: &str, runtime: &mut DebugRuntime) -> String {
    let mut scanner = crate::lexer::scanner::Scanner::new(expr, "<eval>".to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = crate::parser::parser::Parser::new(tokens);

    match parser.parse_expression(0) {
        Ok(parsed_expr) => {
            match crate::interpreter::eval::eval_expr(&parsed_expr, &mut runtime.env) {
                Ok(val) => val.to_string(),
                Err(_) => "<error>".to_string(),
            }
        }
        Err(_) => "<parse error>".to_string(),
    }
}

fn dap_response(request_seq: i64, seq: i64, command: &str, body: Value) -> Value {
    json!({
        "seq": seq,
        "type": "response",
        "request_seq": request_seq,
        "success": true,
        "command": command,
        "body": body
    })
}

fn read_dap_message(reader: &mut impl BufRead) -> Result<Value, String> {
    // Read headers
    let mut content_length = 0;
    loop {
        let mut header = String::new();
        reader.read_line(&mut header).map_err(|e| e.to_string())?;
        let header = header.trim();
        if header.is_empty() { break; }
        if let Some(len_str) = header.strip_prefix("Content-Length:") {
            content_length = len_str.trim().parse().unwrap_or(0);
        }
    }

    if content_length == 0 {
        return Err("empty message".to_string());
    }

    // Read body
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).map_err(|e| e.to_string())?;
    let body_str = String::from_utf8(body).map_err(|e| e.to_string())?;

    serde_json::from_str(&body_str).map_err(|e| e.to_string())
}

fn send_dap_message(writer: &mut impl Write, msg: &Value) {
    let body = serde_json::to_string(msg).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer.write_all(header.as_bytes()).unwrap();
    writer.write_all(body.as_bytes()).unwrap();
    writer.flush().unwrap();
}
