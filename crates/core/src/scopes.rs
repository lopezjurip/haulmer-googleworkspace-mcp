//! T02 — scope catalogue: selection <-> Google scopes, defaults and UI labels.
//! Spec: `docs/spec.md` "Scope catalogue".

use crate::{Level, Selection, Service, ServiceInfo};

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
/// ```
/// use haulmer_connect_core::{scopes, Level, Service};
/// assert!(scopes::default_selection().contains(Service::Gmail, Level::Read));
/// ```
pub fn default_selection() -> Selection {
    Selection::from_levels(
        [
            Service::Gmail,
            Service::Calendar,
            Service::Chat,
            Service::Meet,
        ]
        .map(|service| (service, Level::Read)),
    )
}

/// Normalises a selection: Write implies Read for the same service.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::{scopes, Level, Selection, Service};
/// let normalized = scopes::normalize(&Selection::from_levels([(Service::Gmail, Level::Write)]));
/// assert!(normalized.contains(Service::Gmail, Level::Read));
/// ```
pub fn normalize(selection: &Selection) -> Selection {
    let mut normalized = selection.clone();
    for level in selection.iter().filter(|level| level.level == Level::Write) {
        normalized.insert(level.service, Level::Read);
    }
    normalized
}

/// Full scope URLs for a selection (normalised first), without duplicates.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::{scopes, Level, Selection, Service};
/// let urls = scopes::scopes_for(&Selection::from_levels([(Service::Drive, Level::Read)]));
/// assert_eq!(urls, ["https://www.googleapis.com/auth/drive.readonly"]);
/// ```
pub fn scopes_for(selection: &Selection) -> Vec<String> {
    normalize(selection)
        .iter()
        .flat_map(|level| level_scopes(level.service, level.level))
        .map(|short| format!("{SCOPE_PREFIX}{short}"))
        .collect()
}

/// Short scope names (after [`SCOPE_PREFIX`]) of one service level: the spec table.
/// Levels never share a scope, so concatenating levels yields no duplicates.
///
/// # Example
///
/// ```ignore
/// assert_eq!(level_scopes(Service::Drive, Level::Write), ["drive"]);
/// ```
fn level_scopes(service: Service, level: Level) -> &'static [&'static str] {
    match (service, level) {
        (Service::Gmail, Level::Read) => &["gmail.readonly"],
        (Service::Gmail, Level::Write) => &["gmail.send", "gmail.modify"],
        (Service::Calendar, Level::Read) => &["calendar.readonly"],
        (Service::Calendar, Level::Write) => &["calendar.events"],
        (Service::Chat, Level::Read) => &[
            "chat.spaces.readonly",
            "chat.messages.readonly",
            "chat.memberships.readonly",
        ],
        (Service::Chat, Level::Write) => &["chat.messages"],
        (Service::Meet, Level::Read) => &["meetings.space.readonly"],
        (Service::Meet, Level::Write) => &["meetings.space.created"],
        (Service::Drive, Level::Read) => &["drive.readonly"],
        (Service::Drive, Level::Write) => &["drive"],
    }
}

/// The `--scopes` argument for `gws auth login`: [`scopes_for`] joined with commas.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::{scopes, Level, Selection, Service};
/// let arg = scopes::scopes_arg(&Selection::from_levels([(Service::Calendar, Level::Read)]));
/// assert_eq!(arg, "https://www.googleapis.com/auth/calendar.readonly");
/// ```
pub fn scopes_arg(selection: &Selection) -> String {
    scopes_for(selection).join(",")
}

/// Granted scopes (as `gws auth status` prints them) -> granted levels. A level counts only
/// if all its scopes are granted; identity scopes (`openid`, `email`, `profile`,
/// `userinfo.email`, `userinfo.profile`) are ignored.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::{scopes, Level, Service};
/// let granted = scopes::selection_from_scopes(&["https://www.googleapis.com/auth/drive.readonly".to_string()]);
/// assert!(granted.contains(Service::Drive, Level::Read));
/// ```
pub fn selection_from_scopes(granted: &[String]) -> Selection {
    let is_granted = |short: &&str| {
        granted
            .iter()
            .any(|scope| scope.strip_prefix(SCOPE_PREFIX) == Some(*short))
    };
    let mut selection = Selection::new();
    for service in Service::ALL {
        for level in [Level::Read, Level::Write] {
            if level_scopes(service, level)
                .iter()
                .all(|short| is_granted(&short))
            {
                selection.insert(service, level);
            }
        }
    }
    selection
}

/// Plain-language labels for every service row, in UI order.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::scopes;
/// assert_eq!(scopes::catalogue().len(), 5);
/// ```
pub fn catalogue() -> Vec<ServiceInfo> {
    Service::ALL
        .into_iter()
        .map(|service| {
            let (name, read, write) = match service {
                Service::Gmail => ("Gmail", "Read: read email", "Write: send and modify email"),
                Service::Calendar => (
                    "Calendar",
                    "Read: see calendars and events",
                    "Write: create and edit events",
                ),
                Service::Chat => (
                    "Chat",
                    "Read: read spaces, messages and members",
                    "Write: send messages",
                ),
                Service::Meet => (
                    "Meet",
                    "Read: see meetings and their transcripts",
                    "Write: create meetings",
                ),
                Service::Drive => (
                    "Drive",
                    "Read: see and download files",
                    "Write: create, edit and delete files",
                ),
            };
            ServiceInfo {
                service,
                name,
                read,
                write,
            }
        })
        .collect()
}
