//! MCP server: exposes one tool, `gws`, over the MCP stdio transport
//! (newline-delimited JSON-RPC 2.0). Spec: `docs/spec.md` "MCP".

use std::io::{BufRead, Write};

use serde_json::{Value, json};

use crate::{Core, CoreError};

/// Protocol version answered when the client does not send one.
const DEFAULT_PROTOCOL_VERSION: &str = "2025-06-18";

/// Text of the one tool, shown to the model.
const TOOL_DESCRIPTION: &str = "Run a Google Workspace CLI (gws) command on the user's Google account and return its JSON output. Example args: [\"gmail\",\"+triage\"], [\"calendar\",\"+agenda\"]. Discover methods with [\"schema\",\"gmail.users.messages.list\"].";

/// A JSON-RPC error: code and message.
type RpcError = (i32, &'static str);

/// Serves MCP over `input`/`output` until `input` ends. One JSON-RPC message per line.
/// Requests get one reply line; notifications (no `id`) get none. The `gws` tool runs through
/// the core's runner and refuses `auth` commands (they could export the refresh token).
///
/// # Example
///
/// ```
/// # use haulmer_connect_core::{Core, Paths, mcp};
/// # let dir = tempfile::tempdir().unwrap();
/// # let core = Core::system(Paths { home: dir.path().into(), app_data: dir.path().into() });
/// let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n";
/// let mut output = Vec::new();
/// mcp::serve(&core, input.as_bytes(), &mut output).unwrap();
/// let reply: serde_json::Value = serde_json::from_slice(&output).unwrap();
/// assert_eq!(reply["result"], serde_json::json!({}));
/// ```
pub fn serve(core: &Core, input: impl BufRead, mut output: impl Write) -> Result<(), CoreError> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle(core, &request),
            Err(_) => Some(error_reply(&Value::Null, (-32700, "Parse error"))),
        };
        if let Some(reply) = reply {
            writeln!(output, "{reply}")?;
            output.flush()?;
        }
    }
    Ok(())
}

/// The reply to one request; `None` for a notification (no `id`).
///
/// # Example
///
/// ```ignore
/// let ping = json!({"jsonrpc": "2.0", "id": 7, "method": "ping"});
/// assert_eq!(handle(core, &ping).unwrap()["result"], json!({}));
/// assert!(handle(core, &json!({"method": "notifications/initialized"})).is_none());
/// ```
fn handle(core: &Core, request: &Value) -> Option<Value> {
    let id = request.get("id")?;
    let params = &request["params"];
    let outcome: Result<Value, RpcError> = match request["method"].as_str().unwrap_or("") {
        "initialize" => {
            let protocol_version = params["protocolVersion"]
                .as_str()
                .unwrap_or(DEFAULT_PROTOCOL_VERSION);
            Ok(json!({
                "protocolVersion": protocol_version,
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "haulmer-connect", "version": env!("CARGO_PKG_VERSION")},
            }))
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": [{
            "name": "gws",
            "description": TOOL_DESCRIPTION,
            "inputSchema": {
                "type": "object",
                "properties": {"args": {"type": "array", "items": {"type": "string"}}},
                "required": ["args"],
            },
        }]})),
        "tools/call" => call_tool(core, params),
        _ => Err((-32601, "Method not found")),
    };
    match outcome {
        Ok(result) => Some(json!({"jsonrpc": "2.0", "id": id, "result": result})),
        Err(error) => Some(error_reply(id, error)),
    }
}

/// Runs the `gws` tool: `params` is `{name: "gws", arguments: {args: [...]}}`.
/// Tool failures are results with `isError: true`; only a bad call is a JSON-RPC error.
///
/// # Example
///
/// ```ignore
/// let params = json!({"name": "gws", "arguments": {"args": ["calendar", "+agenda"]}});
/// let result = call_tool(core, &params).unwrap();
/// assert_eq!(result["isError"], false);
/// ```
fn call_tool(core: &Core, params: &Value) -> Result<Value, RpcError> {
    if params["name"] != "gws" {
        return Err((-32602, "Unknown tool"));
    }
    let args: Option<Vec<&str>> = params["arguments"]["args"]
        .as_array()
        .and_then(|list| list.iter().map(Value::as_str).collect());
    let Some(args) = args else {
        return Err((-32602, "args must be an array of strings"));
    };
    if args.first() == Some(&"auth") {
        return Ok(tool_result(
            "auth commands are disabled over MCP; use the haulmer-connect app",
            true,
        ));
    }
    let gws = match core.find_gws() {
        Ok(Some(gws)) => gws,
        Ok(None) => {
            return Ok(tool_result(
                "gws is not installed; open haulmer-connect and click Save",
                true,
            ));
        }
        Err(error) => return Ok(tool_result(&error.to_string(), true)),
    };
    let output = core
        .gws_command(&gws, &args)
        .and_then(|command| core.runner.run(&command));
    match output {
        Ok(output) => {
            let mut text = output.stdout;
            if text.is_empty() {
                text = output.stderr;
            }
            Ok(tool_result(&text, output.exit_code != 0))
        }
        Err(error) => Ok(tool_result(&error.to_string(), true)),
    }
}

/// An MCP tool result with one text block.
///
/// # Example
///
/// ```ignore
/// assert_eq!(tool_result("hi", false)["content"][0]["text"], "hi");
/// ```
fn tool_result(text: &str, is_error: bool) -> Value {
    json!({"content": [{"type": "text", "text": text}], "isError": is_error})
}

/// A JSON-RPC error reply for request `id` (`null` when the request could not be parsed).
///
/// # Example
///
/// ```ignore
/// assert_eq!(error_reply(&json!(1), (-32601, "Method not found"))["error"]["code"], -32601);
/// ```
fn error_reply(id: &Value, (code, message): RpcError) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}
