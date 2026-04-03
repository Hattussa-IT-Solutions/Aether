use std::collections::HashMap;
use std::error::Error;

use lsp_server::{Connection, Message, Request, RequestId, Response, Notification};
use lsp_types::*;
use serde_json::Value;

use crate::lsp::{completions, hover, diagnostics, definition};

/// Start the Aether LSP server (communicates via stdin/stdout).
pub fn run_lsp() -> Result<(), Box<dyn Error>> {
    eprintln!("Aether LSP server starting...");

    let (connection, io_threads) = Connection::stdio();

    // Server capabilities
    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::FULL,
        )),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
            resolve_provider: Some(false),
            ..Default::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    };

    let server_caps = serde_json::to_value(&capabilities)?;
    connection.initialize(server_caps)?;

    eprintln!("Aether LSP initialized");

    // Document store
    let mut documents: HashMap<Url, String> = HashMap::new();

    // Main message loop
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                handle_request(&req, &connection, &documents);
            }
            Message::Notification(notif) => {
                handle_notification(&notif, &mut documents, &connection);
            }
            Message::Response(_) => {}
        }
    }

    io_threads.join()?;
    eprintln!("Aether LSP server stopped");
    Ok(())
}

fn handle_request(
    req: &Request,
    connection: &Connection,
    documents: &HashMap<Url, String>,
) {
    match req.method.as_str() {
        "textDocument/completion" => {
            let params: CompletionParams = serde_json::from_value(req.params.clone()).unwrap();
            let uri = &params.text_document_position.text_document.uri;
            let pos = &params.text_document_position.position;

            let source = documents.get(uri).map(|s| s.as_str()).unwrap_or("");
            let symbols = diagnostics::extract_symbols(source);
            let items = completions::get_completions(source, pos.line, pos.character, &symbols);

            let result = CompletionResponse::Array(items);
            let resp = Response::new_ok(req.id.clone(), serde_json::to_value(result).unwrap());
            connection.sender.send(Message::Response(resp)).unwrap();
        }
        "textDocument/hover" => {
            let params: HoverParams = serde_json::from_value(req.params.clone()).unwrap();
            let uri = &params.text_document_position_params.text_document.uri;
            let pos = &params.text_document_position_params.position;

            let source = documents.get(uri).map(|s| s.as_str()).unwrap_or("");
            let symbols = diagnostics::extract_symbols(source);
            let hover_result = hover::get_hover(source, pos.line, pos.character, &symbols);

            let resp = Response::new_ok(req.id.clone(), serde_json::to_value(hover_result).unwrap());
            connection.sender.send(Message::Response(resp)).unwrap();
        }
        "textDocument/definition" => {
            let params: GotoDefinitionParams = serde_json::from_value(req.params.clone()).unwrap();
            let uri = &params.text_document_position_params.text_document.uri;
            let pos = &params.text_document_position_params.position;

            let source = documents.get(uri).map(|s| s.as_str()).unwrap_or("");
            let def_result = definition::get_definition(source, uri, pos.line, pos.character);

            let resp = Response::new_ok(req.id.clone(), serde_json::to_value(def_result).unwrap());
            connection.sender.send(Message::Response(resp)).unwrap();
        }
        _ => {
            let resp = Response::new_err(req.id.clone(), -32601, "Method not found".to_string());
            connection.sender.send(Message::Response(resp)).unwrap();
        }
    }
}

fn handle_notification(
    notif: &Notification,
    documents: &mut HashMap<Url, String>,
    connection: &Connection,
) {
    match notif.method.as_str() {
        "textDocument/didOpen" => {
            let params: DidOpenTextDocumentParams = serde_json::from_value(notif.params.clone()).unwrap();
            let uri = params.text_document.uri.clone();
            let text = params.text_document.text.clone();
            documents.insert(uri.clone(), text.clone());
            publish_diagnostics(&uri, &text, connection);
        }
        "textDocument/didChange" => {
            let params: DidChangeTextDocumentParams = serde_json::from_value(notif.params.clone()).unwrap();
            let uri = params.text_document.uri.clone();
            if let Some(change) = params.content_changes.into_iter().last() {
                documents.insert(uri.clone(), change.text.clone());
                publish_diagnostics(&uri, &change.text, connection);
            }
        }
        "textDocument/didClose" => {
            let params: DidCloseTextDocumentParams = serde_json::from_value(notif.params.clone()).unwrap();
            documents.remove(&params.text_document.uri);
        }
        _ => {}
    }
}

fn publish_diagnostics(uri: &Url, source: &str, connection: &Connection) {
    let diags = diagnostics::get_diagnostics(uri, source);
    let params = PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics: diags,
        version: None,
    };
    let notif = Notification::new("textDocument/publishDiagnostics".to_string(), serde_json::to_value(params).unwrap());
    connection.sender.send(Message::Notification(notif)).unwrap();
}
