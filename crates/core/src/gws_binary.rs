//! T03 — find or download the pinned gws binary. Spec: `docs/spec.md` "Getting gws".

use std::path::PathBuf;

use crate::{Core, CoreError};

/// The only gws version the app runs.
///
/// # Example
///
/// ```
/// assert_eq!(haulmer_connect_core::gws_binary::GWS_VERSION, "0.22.5");
/// ```
pub const GWS_VERSION: &str = "0.22.5";

impl Core {
    /// The pinned gws to run, without downloading: `gws` on PATH if it reports
    /// [`GWS_VERSION`], else `<app_data>/bin/gws[.exe]` if it does; `None` otherwise.
    /// Never makes HTTP calls.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// let gws = core.find_gws().unwrap(); // Some("gws") or Some("<app_data>/bin/gws") or None
    /// ```
    pub fn find_gws(&self) -> Result<Option<PathBuf>, CoreError> {
        Err(CoreError::NotImplemented("gws_binary::find_gws (T03)"))
    }

    /// Like [`Core::find_gws`], but downloads and verifies the release asset when missing.
    /// A checksum mismatch returns [`CoreError::ChecksumMismatch`] and writes nothing executable.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// let gws = core.ensure_gws().unwrap();
    /// ```
    pub fn ensure_gws(&self) -> Result<PathBuf, CoreError> {
        Err(CoreError::NotImplemented("gws_binary::ensure_gws (T03)"))
    }
}
