use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::rc::Rc;
use tiny_http::{Server, Header, Response as TinyResponse};
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // http_serve(port, handler_fn)
    // Starts an HTTP server on the given port. For each incoming request, calls
    // handler_fn(request_map) -> response_map.
    // request_map:  { method, path, body, headers, query }
    // response_map: { status, body, content_type, headers }
    env.define("http_serve", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "http_serve".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let port = match args.first() {
                Some(Value::Int(n)) => *n as u16,
                Some(Value::Float(f)) => *f as u16,
                _ => return Err("http_serve: first arg must be a port number".into()),
            };
            let handler = match args.get(1) {
                Some(v @ Value::Function(_)) | Some(v @ Value::NativeFunction(_)) => v.clone(),
                _ => return Err("http_serve: second arg must be a handler function".into()),
            };

            let addr = format!("0.0.0.0:{}", port);
            let server = Server::http(&addr)
                .map_err(|e| format!("http_serve: failed to bind {}: {}", addr, e))?;

            println!("http_serve: listening on http://{}", addr);

            // Accept requests in a loop (blocking — runs on the main thread).
            for mut raw_req in server.incoming_requests() {
                // ── Build the request map ──────────────────────────────────────
                let method = raw_req.method().to_string();
                let full_url = raw_req.url().to_string();

                // Split path and query string
                let (path, query) = if let Some(idx) = full_url.find('?') {
                    (full_url[..idx].to_string(), full_url[idx + 1..].to_string())
                } else {
                    (full_url.clone(), String::new())
                };

                // Read body
                let mut body_buf = String::new();
                let _ = raw_req.as_reader().read_to_string(&mut body_buf);

                // Collect headers
                let mut headers_map: HashMap<String, Value> = HashMap::new();
                for h in raw_req.headers() {
                    headers_map.insert(
                        h.field.as_str().as_str().to_lowercase(),
                        Value::String(h.value.as_str().to_string()),
                    );
                }

                let mut req_map: HashMap<String, Value> = HashMap::new();
                req_map.insert("method".to_string(), Value::String(method));
                req_map.insert("path".to_string(), Value::String(path));
                req_map.insert("body".to_string(), Value::String(body_buf));
                req_map.insert("headers".to_string(), Value::Map(Rc::new(RefCell::new(headers_map))));
                req_map.insert("query".to_string(), Value::String(query));

                let req_val = Value::Map(Rc::new(RefCell::new(req_map)));

                // ── Call the Aether handler ───────────────────────────────────
                let resp_val = call_fn_1(&handler, req_val);

                // ── Parse the response map ────────────────────────────────────
                let (status_code, resp_body, content_type, extra_headers) = match resp_val {
                    Ok(Value::Map(m)) => {
                        let mb = m.borrow();
                        let status = mb.get("status")
                            .and_then(|v| v.as_int())
                            .unwrap_or(200) as u16;
                        let body_str = mb.get("body")
                            .map(|v| v.to_string())
                            .unwrap_or_default();
                        let ct = mb.get("content_type")
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "text/plain".to_string());
                        let extra: Vec<(String, String)> = if let Some(Value::Map(hm)) = mb.get("headers") {
                            hm.borrow().iter()
                                .map(|(k, v)| (k.clone(), v.to_string()))
                                .collect()
                        } else {
                            vec![]
                        };
                        (status, body_str, ct, extra)
                    }
                    Ok(other) => (200, other.to_string(), "text/plain".to_string(), vec![]),
                    Err(e)    => (500, e, "text/plain".to_string(), vec![]),
                };

                // ── Build the tiny_http response ──────────────────────────────
                let body_bytes = resp_body.into_bytes();
                let cursor = Cursor::new(body_bytes.clone());
                let mut response = TinyResponse::new(
                    tiny_http::StatusCode(status_code),
                    vec![],
                    cursor,
                    Some(body_bytes.len()),
                    None,
                );

                // Content-Type header
                if let Ok(ct_header) = format!("Content-Type: {}", content_type).parse::<Header>() {
                    response.add_header(ct_header);
                }

                // Extra headers from handler response map
                for (k, v) in extra_headers {
                    if let Ok(h) = format!("{}: {}", k, v).parse::<Header>() {
                        response.add_header(h);
                    }
                }

                let _ = raw_req.respond(response);
            }

            Ok(Value::Nil)
        }),
    })));
}

/// Call a Value::Function or Value::NativeFunction with a single argument.
fn call_fn_1(func: &Value, arg: Value) -> Result<Value, String> {
    match func {
        Value::NativeFunction(nf) => (nf.func)(vec![arg]),
        Value::Function(_) => {
            crate::interpreter::eval::call_function(
                func,
                vec![arg],
                &[],
                &mut crate::interpreter::environment::Environment::new(),
            )
            .map_err(|e| match e {
                Signal::Throw(v)  => v.to_string(),
                Signal::Return(v) => v.to_string(),
                _ => "function error".to_string(),
            })
        }
        _ => Err("expected a function".into()),
    }
}

// We need read_to_string — pull it in.
use std::io::Read;
