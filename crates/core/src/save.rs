//! T07 — `Core::save()` and `Core::cancel_login()`: connect, add and remove services.

use crate::{Core, CoreError, SaveOutcome, Selection};

impl Core {
    /// Applies the selection: may download gws, asks for replace confirmation on foreign
    /// credentials, revokes first when a granted level is removed, then runs the gws login.
    /// Calls `on_auth_url` with the Google sign-in URL (the caller opens the browser).
    /// Blocks until the login ends or [`Core::cancel_login`] is called ([`CoreError::Cancelled`]).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{scopes, Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// let outcome = core.save(&scopes::default_selection(), false, &|url| println!("open {url}"));
    /// ```
    pub fn save(
        &self,
        selection: &Selection,
        confirm_replace: bool,
        on_auth_url: &dyn Fn(&str),
    ) -> Result<SaveOutcome, CoreError> {
        let _ = (selection, confirm_replace, on_auth_url);
        Err(CoreError::NotImplemented("save (T07)"))
    }

    /// Kills the running login, if any; the blocked `save` returns [`CoreError::Cancelled`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// core.cancel_login().unwrap();
    /// ```
    pub fn cancel_login(&self) -> Result<(), CoreError> {
        Err(CoreError::NotImplemented("cancel_login (T07)"))
    }
}
