//! T06 — `Core::revoke_all()`: revoke at Google, then clear local credentials.

use crate::{Core, CoreError};

/// Google's OAuth token revocation endpoint.
///
/// # Example
///
/// ```
/// assert!(haulmer_connect_core::revoke::REVOKE_URL.starts_with("https://oauth2.googleapis.com/"));
/// ```
pub const REVOKE_URL: &str = "https://oauth2.googleapis.com/revoke";

impl Core {
    /// `gws auth export --unmasked` -> POST the refresh token to [`REVOKE_URL`] ->
    /// `gws auth logout` -> remove the ownership marker. If Google fails, still clears
    /// local state and returns [`CoreError::RevokeFailed`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// core.revoke_all().unwrap();
    /// ```
    pub fn revoke_all(&self) -> Result<(), CoreError> {
        Err(CoreError::NotImplemented("revoke_all (T06)"))
    }
}
