//! MCP server: exposes two tools, `gws` and `open_haulmer_connect`, over the MCP stdio
//! transport (newline-delimited JSON-RPC 2.0). Spec: `docs/spec.md` "MCP".

use std::io::{BufRead, Write};

use serde_json::{Value, json};

use crate::{Core, CoreError, Level, Service, scopes};

/// Protocol version answered when the client does not send one.
const DEFAULT_PROTOCOL_VERSION: &str = "2025-06-18";

/// Text of the `gws` tool, shown to the model.
const TOOL_DESCRIPTION: &str = "Run a Google Workspace CLI (gws) command on the user's Google account and return its JSON output. Example args: [\"gmail\",\"+triage\"], [\"calendar\",\"+agenda\"]. Discover methods with [\"schema\",\"gmail.users.messages.list\"]. If a call fails, follow the steps in the error message (it may suggest open_haulmer_connect).";

/// Text of the `open_haulmer_connect` tool, shown to the model.
const OPEN_TOOL_DESCRIPTION: &str = "Open the haulmer-connect desktop app on its settings window, so that the person can connect their Google account or turn on permissions. Takes no arguments. Use it when the person has to act in haulmer-connect, then tell them what to do there.";

/// A JSON-RPC error: code and message.
type RpcError = (i32, &'static str);

/// Serves MCP over `input`/`output` until `input` ends. One JSON-RPC message per line.
/// Requests get one reply line; notifications (no `id`) get none. The `gws` tool runs through
/// the core's runner and refuses `auth` commands (they could export the refresh token); when
/// gws fails, it replies with what the person has to fix. The `open_haulmer_connect` tool
/// calls `open_app`, which the caller provides (the desktop app opens its settings window).
///
/// # Example
///
/// ```
/// # use haulmer_connect_core::{Core, Paths, mcp};
/// # let dir = tempfile::tempdir().unwrap();
/// # let core = Core::system(Paths { home: dir.path().into(), app_data: dir.path().into() });
/// let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n";
/// let mut output = Vec::new();
/// mcp::serve(&core, &|| Ok(()), input.as_bytes(), &mut output).unwrap();
/// let reply: serde_json::Value = serde_json::from_slice(&output).unwrap();
/// assert_eq!(reply["result"], serde_json::json!({}));
/// ```
pub fn serve(
    core: &Core,
    open_app: &dyn Fn() -> Result<(), CoreError>,
    input: impl BufRead,
    mut output: impl Write,
) -> Result<(), CoreError> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle(core, open_app, &request),
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
/// assert_eq!(handle(core, &|| Ok(()), &ping).unwrap()["result"], json!({}));
/// assert!(handle(core, &|| Ok(()), &json!({"method": "notifications/initialized"})).is_none());
/// ```
fn handle(
    core: &Core,
    open_app: &dyn Fn() -> Result<(), CoreError>,
    request: &Value,
) -> Option<Value> {
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
        "tools/list" => Ok(json!({"tools": [
            {
                "name": "gws",
                "description": TOOL_DESCRIPTION,
                "inputSchema": {
                    "type": "object",
                    "properties": {"args": {"type": "array", "items": {"type": "string"}}},
                    "required": ["args"],
                },
            },
            {
                "name": "open_haulmer_connect",
                "description": OPEN_TOOL_DESCRIPTION,
                "inputSchema": {"type": "object", "properties": {}},
            },
        ]})),
        "tools/call" => call_tool(core, open_app, params),
        _ => Err((-32601, "Method not found")),
    };
    match outcome {
        Ok(result) => Some(json!({"jsonrpc": "2.0", "id": id, "result": result})),
        Err(error) => Some(error_reply(id, error)),
    }
}

/// Runs a tool: `params` is `{name: "gws", arguments: {args: [...]}}` or
/// `{name: "open_haulmer_connect", arguments: {}}`.
/// Tool failures are results with `isError: true`; only a bad call is a JSON-RPC error.
///
/// # Example
///
/// ```ignore
/// let params = json!({"name": "gws", "arguments": {"args": ["calendar", "+agenda"]}});
/// let result = call_tool(core, &|| Ok(()), &params).unwrap();
/// assert_eq!(result["isError"], false);
/// ```
fn call_tool(
    core: &Core,
    open_app: &dyn Fn() -> Result<(), CoreError>,
    params: &Value,
) -> Result<Value, RpcError> {
    if params["name"] == "open_haulmer_connect" {
        return match open_app() {
            Ok(()) => Ok(tool_result(
                "Pedí abrir haulmer-connect en su configuración. Si no aparece, pídele a la \
                 persona que lo abra desde el ícono de haulmer-connect (barra de menús en macOS, \
                 bandeja del sistema en Windows) › \"Abrir configuración\".",
                false,
            )),
            Err(error) => Ok(tool_result(&error.to_string(), true)),
        };
    }
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
            "Los comandos auth están desactivados en MCP. Conecta la cuenta desde la app haulmer-connect.",
            true,
        ));
    }
    let gws = match core.find_gws() {
        Ok(Some(gws)) => gws,
        Ok(None) => {
            return Ok(tool_result(&gws_missing_text(), true));
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
            if output.exit_code != 0 {
                text = diagnose(core, &args, text);
            }
            Ok(tool_result(&text, output.exit_code != 0))
        }
        Err(error) => Ok(tool_result(&error.to_string(), true)),
    }
}

/// What to tell the agent when a gws call failed.
///
/// It reports the first problem found in the status of the connection. The problems, in order,
/// are: gws missing, no account, expired session, service not granted. When the connection is
/// fine or cannot be read, it returns `raw` (the gws output). A service granted at the Read level
/// only adds a hint about the Write level.
///
/// # Example
///
/// ```ignore
/// // No Google account connected:
/// let text = diagnose(core, &["drive", "files", "list"], "no credentials".to_string());
/// assert!(text.starts_with("No hay una cuenta de Google conectada."));
/// ```
fn diagnose(core: &Core, args: &[&str], raw: String) -> String {
    let Ok(status) = core.status() else {
        return raw;
    };
    if !status.gws_available {
        return gws_missing_text();
    }
    if status.account.is_none() {
        return fix_in_settings(
            "No hay una cuenta de Google conectada.",
            "pulse \"Conectar\" e inicie sesión con Google",
        );
    }
    if !status.token_valid {
        return fix_in_settings(
            "La sesión de Google expiró.",
            "pulse \"Guardar\" para iniciar sesión de nuevo",
        );
    }
    let service = Service::ALL
        .into_iter()
        .find(|service| Some(service.slug()) == args.first().copied());
    let Some(service) = service else {
        return raw;
    };
    let name = scopes::catalogue()
        .into_iter()
        .find(|info| info.service == service)
        .map_or(service.slug(), |info| info.name);
    if !status.granted.iter().any(|level| level.service == service) {
        return fix_in_settings(
            &format!("Falta el permiso de {name}."),
            &format!("active {name} y pulse \"Guardar\""),
        );
    }
    if !status.granted.contains(service, Level::Write) {
        return format!(
            "{raw}\n\n{}",
            fix_in_settings(
                &format!(
                    "{name} solo tiene el nivel de lectura. Si esta acción cambia datos, quizá \
                     falte el nivel de Escritura."
                ),
                &format!("active \"Escritura\" en {name} y pulse \"Guardar\""),
            )
        );
    }
    raw
}

/// The Spanish text for the agent when gws is not installed, with the ways to fix it.
///
/// # Example
///
/// ```ignore
/// assert!(gws_missing_text().starts_with("gws no está instalado."));
/// ```
fn gws_missing_text() -> String {
    fix_in_settings(
        "gws no está instalado.",
        "pulse \"Conectar\" (descarga gws e inicia sesión con Google)",
    )
}

/// A problem plus the two ways to fix it, in Spanish, for the agent to relay: ask the person
/// to do `action` in the settings window, or open that window for them.
///
/// # Example
///
/// ```ignore
/// let text = fix_in_settings("La sesión de Google expiró.", "pulse \"Guardar\"");
/// assert!(text.contains("open_haulmer_connect"));
/// ```
fn fix_in_settings(problem: &str, action: &str) -> String {
    format!(
        "{problem} Haz una de estas dos cosas:\n\
         1. Pídele a la persona que abra haulmer-connect (su ícono en la barra de menú de macOS, \
         o en la bandeja del sistema de Windows, junto al reloj) › \"Abrir configuración\", \
         y que {action}.\n\
         2. O llama a la herramienta open_haulmer_connect para abrir esa ventana por ella."
    )
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
