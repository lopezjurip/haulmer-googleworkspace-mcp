//! The shared haulmer-googleworkspace-mcp OAuth client (Desktop app, Internal audience).
//!
//! This is a public client (RFC 6749 §2.1, RFC 8252): installed apps cannot keep
//! credentials confidential. OAuth uses the client ID and PKCE.
//!
//! # Example
//!
//! ```
//! use haulmer_googleworkspace_mcp_core::oauth_client::CLIENT_ID;
//! assert!(CLIENT_ID.ends_with(".apps.googleusercontent.com"));
//! ```

/// OAuth client ID of the Google Cloud project `haulmer-googleworkspace-mcp`
/// (project ID `haulmer-connect`).
pub const CLIENT_ID: &str =
    "252304543508-fu07n7mngakrpuvmvdo28rek975lkccn.apps.googleusercontent.com";
