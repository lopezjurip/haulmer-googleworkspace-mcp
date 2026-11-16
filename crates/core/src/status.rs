//! T05 — `Core::status()`: what is connected.

use crate::{Core, CoreError, Status};

impl Core {
    /// Reports gws availability (no download), the connected account, granted levels,
    /// the active config folder, key storage and foreign credentials.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// let status = core.status().unwrap();
    /// println!("{:?}", status.account);
    /// ```
    pub fn status(&self) -> Result<Status, CoreError> {
        Err(CoreError::NotImplemented("status (T05)"))
    }
}
