//! The Install prompt: the Spanish text a person pastes into an Agent (Claude Code, Codex, ...)
//! so that the Agent registers `haulmer-connect mcp` as a local MCP server by itself.
//! Spec: `docs/spec.md` "MCP".

use std::path::Path;

use serde::Serialize;
use serde_json::json;

use crate::CoreError;
use crate::skills::skills_add_command;

/// The text to copy, plus a warning to show when the path may not last.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::mcp_clients::InstallPrompt;
/// let prompt = InstallPrompt { text: "...".into(), warning: None };
/// assert_eq!(serde_json::to_string(&prompt).unwrap(), r#"{"text":"...","warning":null}"#);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct InstallPrompt {
    /// The prompt, in Spanish.
    pub text: String,
    /// A Spanish warning for the person; the prompt is still usable.
    pub warning: Option<String>,
}

/// The Spanish Install prompt with `exe` as the MCP server command, and the core service skills
/// ([`skills_add_command`]) as an optional last step.
///
/// Apple has no supported API to detect app translocation. So this function reads the path text.
/// It fails for a translocated path. It warns for `/Volumes/`, which is the mounted disk image or
/// a working external drive. It uses `exe` as given. Never canonicalize it: on Windows that
/// returns `\\?\` paths.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::mcp_clients::install_prompt;
/// let installed = "/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect";
/// let prompt = install_prompt(installed.as_ref()).unwrap();
/// assert!(prompt.text.contains(&format!("-- \"{installed}\" mcp")));
/// assert!(prompt.warning.is_none());
///
/// let translocated = "/private/var/folders/x/T/AppTranslocation/AB12/d/haulmer-connect.app/Contents/MacOS/haulmer-connect";
/// assert!(install_prompt(translocated.as_ref()).is_err());
/// ```
pub fn install_prompt(exe: &Path) -> Result<InstallPrompt, CoreError> {
    let exe = exe.to_string_lossy();
    if exe.contains("/AppTranslocation/") {
        return Err(CoreError::UnstableExe(
            "macOS está ejecutando haulmer-connect desde una carpeta temporal. Muévela a \
             Aplicaciones, ábrela de nuevo y vuelve a abrir esta ventana."
                .to_string(),
        ));
    }
    let mut warning = None;
    if exe.starts_with("/Volumes/") {
        warning = Some(
            "Si abriste haulmer-connect desde la imagen de disco (.dmg), muévela a Aplicaciones \
             y ábrela de nuevo: esta ruta deja de existir al expulsar el disco."
                .to_string(),
        );
    }

    let desktop_json = serde_json::to_string_pretty(&json!({
        "mcpServers": {"haulmer-connect": {"command": exe, "args": ["mcp"]}}
    }))?;

    let text = format!(
        include_str!("install_prompt.es.txt"),
        exe = exe,
        desktop_json = desktop_json,
        skills_command = skills_add_command(),
    );
    Ok(InstallPrompt { text, warning })
}
