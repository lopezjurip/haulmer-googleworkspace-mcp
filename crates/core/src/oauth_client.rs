//! The shared haulmer-connect OAuth client (Desktop app, Internal audience).
//!
//! This is a public client (RFC 6749 §2.1, RFC 8252): the ID and secret ship inside the app
//! and in the public repository and are not treated as secrets (RFC 8252 §8.5). Only
//! haulmer.com accounts can consent, and a Desktop client only redirects to loopback.
//! gws receives them through `GOOGLE_WORKSPACE_CLI_CLIENT_ID` / `_CLIENT_SECRET`.
//!
//! # Example
//!
//! ```
//! use haulmer_connect_core::oauth_client::{CLIENT_ID, CLIENT_SECRET};
//! assert!(CLIENT_ID.ends_with(".apps.googleusercontent.com"));
//! assert!(!CLIENT_SECRET.is_empty());
//! ```

/// OAuth client ID of the Google Cloud project `haulmer-connect`.
pub const CLIENT_ID: &str =
    "252304543508-fu07n7mngakrpuvmvdo28rek975lkccn.apps.googleusercontent.com";

/// OAuth client secret of the Desktop client (public by design, RFC 8252 §8.5).
pub const CLIENT_SECRET: &str = "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";
