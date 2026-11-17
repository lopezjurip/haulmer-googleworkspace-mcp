//! MCP server (`mcp::serve`) and the client config writers (`Core::add_to_*`).

use std::path::PathBuf;
use std::sync::Arc;

use haulmer_connect_core::testing::{FakeHttp, FakeReply, FakeRunner};
use haulmer_connect_core::{Core, Paths, mcp};
use serde_json::{Value, json};
use tempfile::TempDir;

/// A core on temp dirs with a fake runner; gws 0.22.5 is on PATH.
fn setup() -> (TempDir, TempDir, Arc<FakeRunner>, Core) {
    let (home, app_data) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let runner = Arc::new(FakeRunner::new());
    runner.on_program("gws", &["--version"], FakeReply::ok("gws 0.22.5\n"));
    let paths = Paths {
        home: home.path().into(),
        app_data: app_data.path().into(),
    };
    let core = Core::new(paths, runner.clone(), Arc::new(FakeHttp::new()));
    (home, app_data, runner, core)
}

/// Sends each request as one line and returns the reply lines parsed.
fn serve(core: &Core, requests: &[Value]) -> Vec<Value> {
    let input: String = requests
        .iter()
        .map(|request| format!("{request}\n"))
        .collect();
    let mut output = Vec::new();
    mcp::serve(core, input.as_bytes(), &mut output).unwrap();
    String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn call(id: u32, args: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": "tools/call",
        "params": {"name": "gws", "arguments": {"args": args}}})
}

#[test]
fn initialize_and_list_tools() {
    let (_home, _app_data, _runner, core) = setup();
    let replies = serve(
        &core,
        &[
            json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": "2025-03-26"}}),
            json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
            json!({"jsonrpc": "2.0", "id": 3, "method": "nope"}),
        ],
    );
    assert_eq!(replies.len(), 3); // the notification got no reply
    assert_eq!(replies[0]["result"]["protocolVersion"], "2025-03-26");
    assert_eq!(
        replies[0]["result"]["serverInfo"]["name"],
        "haulmer-connect"
    );
    assert_eq!(replies[1]["result"]["tools"][0]["name"], "gws");
    assert_eq!(
        replies[1]["result"]["tools"][0]["inputSchema"]["required"],
        json!(["args"])
    );
    assert_eq!(replies[2]["error"]["code"], -32601);
}

#[test]
fn bad_json_is_a_parse_error() {
    let (_home, _app_data, _runner, core) = setup();
    let mut output = Vec::new();
    mcp::serve(&core, "{nope\n".as_bytes(), &mut output).unwrap();
    let reply: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(reply["error"]["code"], -32700);
}

#[test]
fn tool_call_runs_gws_and_returns_stdout() {
    let (_home, _app_data, runner, core) = setup();
    runner.on(&["gmail", "+triage"], FakeReply::ok("{\"unread\":3}"));
    let replies = serve(&core, &[call(1, json!(["gmail", "+triage"]))]);
    assert_eq!(replies[0]["result"]["content"][0]["text"], "{\"unread\":3}");
    assert_eq!(replies[0]["result"]["isError"], false);
    assert!(
        runner
            .args()
            .contains(&vec!["gmail".to_string(), "+triage".to_string()])
    );
}

#[test]
fn tool_call_reports_failure_from_stderr() {
    let (_home, _app_data, runner, core) = setup();
    runner.on(&["drive"], FakeReply::exit(2, "", "no credentials"));
    let replies = serve(&core, &[call(1, json!(["drive", "files"]))]);
    assert_eq!(replies[0]["result"]["content"][0]["text"], "no credentials");
    assert_eq!(replies[0]["result"]["isError"], true);
}

#[test]
fn tool_call_refuses_auth() {
    let (_home, _app_data, runner, core) = setup();
    let replies = serve(&core, &[call(1, json!(["auth", "export", "--unmasked"]))]);
    assert_eq!(replies[0]["result"]["isError"], true);
    assert_eq!(
        replies[0]["result"]["content"][0]["text"],
        "auth commands are disabled over MCP; use the haulmer-connect app"
    );
    assert!(runner.calls().is_empty());
}

fn claude_config(home: &TempDir) -> PathBuf {
    let base = if cfg!(target_os = "macos") {
        "Library/Application Support"
    } else if cfg!(windows) {
        "AppData/Roaming"
    } else {
        ".config"
    };
    home.path()
        .join(base)
        .join("Claude/claude_desktop_config.json")
}

#[test]
fn claude_config_keeps_other_servers() {
    let (home, _app_data, _runner, core) = setup();
    let path = claude_config(&home);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        r#"{"theme":"dark","mcpServers":{"other":{"command":"x","args":[]}}}"#,
    )
    .unwrap();

    let written = core
        .add_to_claude_desktop("/bin/haulmer-connect".as_ref())
        .unwrap();

    assert_eq!(written, path);
    let config: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(config["theme"], "dark");
    assert_eq!(config["mcpServers"]["other"]["command"], "x");
    assert_eq!(
        config["mcpServers"]["haulmer-connect"],
        json!({"command": "/bin/haulmer-connect", "args": ["mcp"]})
    );
}

#[test]
fn claude_config_creates_missing_file() {
    let (home, _app_data, _runner, core) = setup();
    core.add_to_claude_desktop("/bin/haulmer-connect".as_ref())
        .unwrap();
    assert!(claude_config(&home).is_file());
}

#[test]
fn claude_config_with_invalid_json_is_not_overwritten() {
    let (home, _app_data, _runner, core) = setup();
    let path = claude_config(&home);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "{ not json").unwrap();

    assert!(
        core.add_to_claude_desktop("/bin/haulmer-connect".as_ref())
            .is_err()
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ not json");
}

#[test]
fn chatgpt_config_appends_and_keeps_other_content() {
    let (home, _app_data, _runner, core) = setup();
    let path = home.path().join(".codex/config.toml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        "model = \"gpt-5\"\n\n[mcp_servers.other]\ncommand = \"x\"\n",
    )
    .unwrap();

    core.add_to_chatgpt("/bin/haulmer-connect".as_ref())
        .unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("model = \"gpt-5\"\n\n[mcp_servers.other]\ncommand = \"x\"\n"));
    assert!(text.ends_with(
        "\n[mcp_servers.haulmer-connect]\ncommand = \"/bin/haulmer-connect\"\nargs = [\"mcp\"]\n"
    ));
}

#[test]
fn chatgpt_config_twice_leaves_one_table() {
    let (home, _app_data, _runner, core) = setup();
    let path = home.path().join(".codex/config.toml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "[mcp_servers.other]\ncommand = \"x\"\n").unwrap();

    core.add_to_chatgpt("/old/haulmer-connect".as_ref())
        .unwrap();
    core.add_to_chatgpt("C:\\new\\haulmer-connect.exe".as_ref())
        .unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(text.matches("[mcp_servers.haulmer-connect]").count(), 1);
    assert!(text.contains("command = \"C:\\\\new\\\\haulmer-connect.exe\""));
    assert!(!text.contains("/old/"));
    assert!(text.contains("[mcp_servers.other]\ncommand = \"x\"\n"));
}
