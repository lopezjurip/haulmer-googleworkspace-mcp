//! T03 — the real [`HttpClient`] on ureq (rustls, follows redirects).

use crate::{CoreError, HttpClient, HttpResponse};

/// Largest body read (the gws archives are well below this).
const MAX_BODY: u64 = 200 * 1024 * 1024;

/// An agent that returns every HTTP status as a response instead of an error.
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into()
}

/// Reads a ureq result into an [`HttpResponse`]; transport failures become [`CoreError::Http`].
fn to_response(
    url: &str,
    result: Result<ureq::http::Response<ureq::Body>, ureq::Error>,
) -> Result<HttpResponse, CoreError> {
    let http_error = |error: ureq::Error| CoreError::Http(format!("{url}: {error}"));
    let mut response = result.map_err(http_error)?;
    let status = response.status().as_u16();
    let body = response
        .body_mut()
        .with_config()
        .limit(MAX_BODY)
        .read_to_vec()
        .map_err(http_error)?;
    Ok(HttpResponse::new(status, body))
}

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
    /// GETs a URL, following redirects.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use haulmer_connect_core::{HttpClient, SystemHttp};
    /// let response = SystemHttp::new().get("https://example.com");
    /// ```
    fn get(&self, url: &str) -> Result<HttpResponse, CoreError> {
        to_response(url, agent().get(url).call())
    }

    fn get_bearer(&self, url: &str, token: &str) -> Result<HttpResponse, CoreError> {
        to_response(
            url,
            agent()
                .get(url)
                .header("Authorization", &format!("Bearer {token}"))
                .call(),
        )
    }

    /// POSTs an `application/x-www-form-urlencoded` form.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use haulmer_connect_core::{HttpClient, SystemHttp};
    /// let response = SystemHttp::new().post_form("https://oauth2.googleapis.com/revoke", &[("token", "t")]);
    /// ```
    fn post_form(&self, url: &str, form: &[(&str, &str)]) -> Result<HttpResponse, CoreError> {
        to_response(url, agent().post(url).send_form(form.iter().copied()))
    }
}
