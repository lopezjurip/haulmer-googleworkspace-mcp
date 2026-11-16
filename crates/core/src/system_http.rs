//! T03 — the real [`HttpClient`] (ureq, added by T03).

use crate::{CoreError, HttpClient, HttpResponse};

/// Real HTTP client used by the app. Must follow redirects and return non-2xx statuses as `Ok`.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::SystemHttp;
/// let http = SystemHttp::new();
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemHttp;

impl SystemHttp {
    /// Creates the client.
    ///
    /// # Example
    ///
    /// ```
    /// let http = haulmer_connect_core::SystemHttp::new();
    /// ```
    pub fn new() -> Self {
        Self
    }
}

impl HttpClient for SystemHttp {
    /// GETs a URL (stub until T03).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use haulmer_connect_core::{HttpClient, SystemHttp};
    /// let response = SystemHttp::new().get("https://example.com");
    /// ```
    fn get(&self, url: &str) -> Result<HttpResponse, CoreError> {
        let _ = url;
        Err(CoreError::NotImplemented("system_http::get (T03)"))
    }

    /// POSTs a form (stub until T03).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use haulmer_connect_core::{HttpClient, SystemHttp};
    /// let response = SystemHttp::new().post_form("https://oauth2.googleapis.com/revoke", &[("token", "t")]);
    /// ```
    fn post_form(&self, url: &str, form: &[(&str, &str)]) -> Result<HttpResponse, CoreError> {
        let _ = (url, form);
        Err(CoreError::NotImplemented("system_http::post_form (T03)"))
    }
}
