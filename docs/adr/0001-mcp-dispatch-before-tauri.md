# The `mcp` subcommand is dispatched in main.rs, before Tauri starts

`haulmer-connect mcp` must serve clean JSON-RPC on stdout and open no window. `tauri-plugin-cli` parses arguments only after `Builder::build()`, when the webviews already exist, and it cannot report the executable path, so it does not help the install prompt either. We keep the hand-written check in `src-tauri/src/main.rs` instead. Evidence: `docs/research/mcp-install-prompt.md`, section 2.
