//! Writes the haulmer-connect MCP server into the config of desktop AI apps.
//! Spec: `docs/spec.md` "MCP".

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::{Core, CoreError};

/// Name of the server entry in every client config.
const SERVER_NAME: &str = "haulmer-connect";

/// Reads `path` as text; a missing file is an empty string.
///
/// # Example
///
/// ```ignore
/// assert_eq!(read_or_empty("/nonexistent".as_ref()).unwrap(), "");
/// ```
fn read_or_empty(path: &Path) -> Result<String, CoreError> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error.into()),
    }
}

/// Writes `text` to `path`, creating the parent folder first.
///
/// # Example
///
/// ```ignore
/// write_with_parent("/tmp/x/config.toml".as_ref(), "a = 1\n").unwrap();
/// ```
fn write_with_parent(path: &Path, text: &str) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(())
}

impl Core {
    /// Adds `mcpServers["haulmer-connect"] = {command: exe, args: ["mcp"]}` to Claude Desktop's
    /// config (macOS `<home>/Library/Application Support/Claude`, Windows
    /// `<home>\AppData\Roaming\Claude`, else `<home>/.config/Claude`), keeping every other key.
    /// A config that is not valid JSON is left untouched and returns [`CoreError::Parse`].
    /// Returns the config file path.
    ///
    /// # Example
    ///
    /// ```
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let home = tempfile::tempdir().unwrap();
    /// # let core = Core::system(Paths { home: home.path().into(), app_data: "/a".into() });
    /// let path = core.add_to_claude_desktop("/Applications/haulmer-connect".as_ref()).unwrap();
    /// assert!(path.ends_with("claude_desktop_config.json"));
    /// ```
    pub fn add_to_claude_desktop(&self, exe: &Path) -> Result<PathBuf, CoreError> {
        let mut folder = self.paths.home.clone();
        if cfg!(target_os = "macos") {
            folder.push("Library/Application Support");
        } else if cfg!(windows) {
            folder.push("AppData/Roaming");
        } else {
            folder.push(".config");
        }
        let path = folder.join("Claude").join("claude_desktop_config.json");

        let text = read_or_empty(&path)?;
        let mut config = json!({});
        if !text.trim().is_empty() {
            config = serde_json::from_str(&text).map_err(|error| {
                CoreError::Parse(format!("{} is not valid JSON: {error}", path.display()))
            })?;
        }
        let not_object = || CoreError::Parse(format!("{} is not a JSON object", path.display()));
        let root = config.as_object_mut().ok_or_else(not_object)?;
        let servers = root.entry("mcpServers").or_insert_with(|| json!({}));
        let servers = servers.as_object_mut().ok_or_else(not_object)?;
        servers.insert(
            SERVER_NAME.to_string(),
            json!({"command": exe.to_string_lossy(), "args": ["mcp"]}),
        );

        write_with_parent(&path, &(serde_json::to_string_pretty(&config)? + "\n"))?;
        Ok(path)
    }

    /// Writes the `[mcp_servers.haulmer-connect]` table into `<home>/.codex/config.toml`, the
    /// file shared by the ChatGPT desktop app, the Codex CLI and the Codex IDE extension
    /// (<https://learn.chatgpt.com/docs/extend/mcp>). An existing table is replaced (up to the
    /// next `[` header line); otherwise it is appended. Everything else is kept.
    /// Returns the config file path.
    ///
    /// # Example
    ///
    /// ```
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let home = tempfile::tempdir().unwrap();
    /// # let core = Core::system(Paths { home: home.path().into(), app_data: "/a".into() });
    /// let path = core.add_to_chatgpt("/Applications/haulmer-connect".as_ref()).unwrap();
    /// assert!(std::fs::read_to_string(path).unwrap().contains("[mcp_servers.haulmer-connect]"));
    /// ```
    pub fn add_to_chatgpt(&self, exe: &Path) -> Result<PathBuf, CoreError> {
        let path = self.paths.home.join(".codex").join("config.toml");
        let text = read_or_empty(&path)?;

        let exe = exe
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        let table = [
            format!("[mcp_servers.{SERVER_NAME}]"),
            format!("command = \"{exe}\""),
            "args = [\"mcp\"]".to_string(),
        ];

        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        let start = lines.iter().position(|line| line.trim() == table[0]);
        match start {
            Some(start) => {
                let mut end = lines.len();
                if let Some(next) = lines[start + 1..]
                    .iter()
                    .position(|line| line.trim_start().starts_with('['))
                {
                    end = start + 1 + next;
                }
                // Keep the blank lines that separate this table from the next one.
                while end > start + 1 && lines[end - 1].trim().is_empty() {
                    end -= 1;
                }
                lines.splice(start..end, table);
            }
            None => {
                if !lines.is_empty() {
                    lines.push(String::new());
                }
                lines.extend(table);
            }
        }

        write_with_parent(&path, &(lines.join("\n") + "\n"))?;
        Ok(path)
    }
}
