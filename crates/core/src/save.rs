//! T07 — `Core::save()` and `Core::cancel_login()`: connect, add and remove services.

use std::path::Path;

use crate::{Core, CoreError, OutputLine, SaveOutcome, Selection, gws_env, scopes};

impl Core {
    /// Applies the selection: may download gws, asks for replace confirmation on foreign
    /// credentials, revokes first when a granted level is removed, then runs the gws login.
    /// An empty selection revokes everything (like [`Core::revoke_all`]) and signs in to nothing.
    /// A foreign token is never revoked: with `confirm_replace` the login overwrites it.
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
        let config_dir = self.settings()?.active_config_dir(&self.paths);
        let foreign = gws_env::foreign_credentials(&config_dir);
        if foreign && !confirm_replace {
            return Ok(SaveOutcome::NeedsReplaceConfirmation);
        }
        let selection = scopes::normalize(selection);
        if selection.is_empty() {
            if !foreign {
                self.revoke_all()?;
            }
            return Ok(SaveOutcome::Connected(self.status()?));
        }
        let gws = self.ensure_gws()?;
        let granted = if foreign {
            Selection::new()
        } else {
            self.status()?.granted
        };
        if granted
            .iter()
            .any(|level| !selection.contains(level.service, level.level))
        {
            self.revoke_and_logout(&gws)?;
        }
        self.login(&gws, &selection, on_auth_url)?;
        gws_env::write_marker(&config_dir)?;
        Ok(SaveOutcome::Connected(self.status()?))
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
        let killer = self.active_login.lock().unwrap().take();
        if let Some(killer) = killer {
            killer.kill();
        }
        Ok(())
    }

    /// Runs `gws auth login --scopes <urls>`, forwards the first sign-in URL and waits.
    /// [`Core::cancel_login`] taking the killer while it runs means it was cancelled.
    fn login(
        &self,
        gws: &Path,
        selection: &Selection,
        on_auth_url: &dyn Fn(&str),
    ) -> Result<(), CoreError> {
        let scopes = scopes::scopes_arg(selection);
        let command = self.gws_command(gws, &["auth", "login", "--scopes", &scopes])?;
        let mut process = self.runner.spawn(&command)?;
        *self.active_login.lock().unwrap() = Some(process.killer());

        let (mut stdout, mut url_sent) = (String::new(), false);
        while let Some(line) = process.next_line() {
            let text = match line {
                OutputLine::Stdout(text) => {
                    stdout.push_str(&text);
                    stdout.push('\n');
                    text
                }
                OutputLine::Stderr(text) => text,
            };
            if !url_sent && let Some(url) = auth_url(&text) {
                url_sent = true;
                on_auth_url(url);
            }
        }
        let exit_code = process.wait();
        let cancelled = self.active_login.lock().unwrap().take().is_none();
        let exit_code = exit_code?;
        if exit_code == 0 {
            return Ok(());
        }
        if cancelled {
            return Err(CoreError::Cancelled);
        }
        Err(CoreError::Gws {
            exit_code,
            message: gws_error_message(&stdout),
        })
    }
}

/// The Google sign-in URL in a gws output line, without the surrounding text.
fn auth_url(line: &str) -> Option<&str> {
    let start = line.find("https://accounts.google.com/")?;
    line[start..].split_whitespace().next()
}

/// `error.message` from gws's JSON error on stdout, else the trimmed stdout.
fn gws_error_message(stdout: &str) -> String {
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap_or_default();
    match json["error"]["message"].as_str() {
        Some(message) => message.to_string(),
        None => stdout.trim().to_string(),
    }
}
