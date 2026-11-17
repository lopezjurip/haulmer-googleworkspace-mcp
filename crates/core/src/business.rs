//! Shared-OAuth business connectors.
//!
//! The first connector is the authenticated YouTube channel lookup. It uses the
//! refresh token already managed by `gws`, then keeps the access token in memory.

use std::path::Path;

use serde_json::Value;

use crate::oauth_client::{CLIENT_ID, CLIENT_SECRET};
use crate::{Core, CoreError};

const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const YOUTUBE_CHANNELS_URL: &str = "https://www.googleapis.com/youtube/v3/channels?part=snippet,contentDetails,statistics&mine=true";

impl Core {
    /// Returns the authenticated user's YouTube channel resource.
    ///
    /// The refresh token is read through `gws auth export --unmasked`, exchanged
    /// for a short-lived access token, and used only in an Authorization header.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// let channel = core.youtube_channel()?;
    /// println!("{}", channel["items"][0]["snippet"]["title"]);
    /// # Ok::<(), haulmer_connect_core::CoreError>(())
    /// ```
    pub fn youtube_channel(&self) -> Result<Value, CoreError> {
        let gws = self.find_gws()?.ok_or_else(|| CoreError::Gws {
            exit_code: 2,
            message: "gws is not installed; open haulmer-connect and click Save".into(),
        })?;
        let refresh_token = self.refresh_token(&gws)?;
        let access_token = self.exchange_refresh_token(&refresh_token)?;
        let response = self.http.get_bearer(YOUTUBE_CHANNELS_URL, &access_token)?;
        if response.status != 200 {
            return Err(CoreError::Http(format!(
                "YouTube answered HTTP {}: {}",
                response.status,
                String::from_utf8_lossy(&response.body)
            )));
        }
        Ok(serde_json::from_slice(&response.body)?)
    }

    fn refresh_token(&self, gws: &Path) -> Result<String, CoreError> {
        let output = self
            .runner
            .run(&self.gws_command(gws, &["auth", "export", "--unmasked"])?)?;
        let json: Value = serde_json::from_str(&output.stdout)?;
        json["refresh_token"]
            .as_str()
            .filter(|_| output.exit_code == 0)
            .map(str::to_owned)
            .ok_or_else(|| CoreError::Gws {
                exit_code: output.exit_code,
                message: "could not read the refresh token".into(),
            })
    }

    fn exchange_refresh_token(&self, refresh_token: &str) -> Result<String, CoreError> {
        let response = self.http.post_form(
            TOKEN_URL,
            &[
                ("client_id", CLIENT_ID),
                ("client_secret", CLIENT_SECRET),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ],
        )?;
        if response.status != 200 {
            return Err(CoreError::Http(format!(
                "Google token endpoint answered HTTP {}",
                response.status
            )));
        }
        let json: Value = serde_json::from_slice(&response.body)?;
        json["access_token"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| CoreError::Parse("Google token response has no access_token".into()))
    }
}
