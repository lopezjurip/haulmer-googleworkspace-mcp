//! T02 — scope catalogue: selection <-> Google scopes, defaults and UI labels.
//! Spec: `docs/spec.md` "Scope catalogue".

use crate::{Selection, ServiceInfo};

/// Prefix of every Google scope URL in the catalogue.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::scopes::SCOPE_PREFIX;
/// assert_eq!(format!("{SCOPE_PREFIX}drive"), "https://www.googleapis.com/auth/drive");
/// ```
pub const SCOPE_PREFIX: &str = "https://www.googleapis.com/auth/";

/// Default selection: Read for Gmail, Calendar, Chat and Meet; Drive and every Write off.
///
/// # Example
///
/// ```should_panic
/// use haulmer_connect_core::{scopes, Level, Service};
/// assert!(scopes::default_selection().contains(Service::Gmail, Level::Read));
/// ```
pub fn default_selection() -> Selection {
    todo!("T02")
}

/// Normalises a selection: Write implies Read for the same service.
///
/// # Example
///
/// ```should_panic
/// use haulmer_connect_core::{scopes, Level, Selection, Service};
/// let normalized = scopes::normalize(&Selection::from_levels([(Service::Gmail, Level::Write)]));
/// assert!(normalized.contains(Service::Gmail, Level::Read));
/// ```
pub fn normalize(selection: &Selection) -> Selection {
    let _ = selection;
    todo!("T02")
}

/// Full scope URLs for a selection (normalised first), without duplicates.
///
/// # Example
///
/// ```should_panic
/// use haulmer_connect_core::{scopes, Level, Selection, Service};
/// let urls = scopes::scopes_for(&Selection::from_levels([(Service::Drive, Level::Read)]));
/// assert_eq!(urls, ["https://www.googleapis.com/auth/drive.readonly"]);
/// ```
pub fn scopes_for(selection: &Selection) -> Vec<String> {
    let _ = selection;
    todo!("T02")
}

/// The `--scopes` argument for `gws auth login`: [`scopes_for`] joined with commas.
///
/// # Example
///
/// ```should_panic
/// use haulmer_connect_core::{scopes, Level, Selection, Service};
/// let arg = scopes::scopes_arg(&Selection::from_levels([(Service::Calendar, Level::Read)]));
/// assert_eq!(arg, "https://www.googleapis.com/auth/calendar.readonly");
/// ```
pub fn scopes_arg(selection: &Selection) -> String {
    let _ = selection;
    todo!("T02")
}

/// Granted scopes (as `gws auth status` prints them) -> granted levels. A level counts only
/// if all its scopes are granted; identity scopes (`openid`, `email`, `profile`,
/// `userinfo.email`, `userinfo.profile`) are ignored.
///
/// # Example
///
/// ```should_panic
/// use haulmer_connect_core::{scopes, Level, Service};
/// let granted = scopes::selection_from_scopes(&["https://www.googleapis.com/auth/drive.readonly".to_string()]);
/// assert!(granted.contains(Service::Drive, Level::Read));
/// ```
pub fn selection_from_scopes(granted: &[String]) -> Selection {
    let _ = granted;
    todo!("T02")
}

/// Plain-language labels for every service row, in UI order.
///
/// # Example
///
/// ```should_panic
/// use haulmer_connect_core::scopes;
/// assert_eq!(scopes::catalogue().len(), 5);
/// ```
pub fn catalogue() -> Vec<ServiceInfo> {
    todo!("T02")
}
