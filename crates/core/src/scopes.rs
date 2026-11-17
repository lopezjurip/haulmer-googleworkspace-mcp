//! T02 — scope catalogue: selection <-> Google scopes, defaults and UI labels.
//! Spec: `docs/spec.md` "Scope catalogue".

use crate::{
    BusinessConnector, BusinessConnectorInfo, BusinessSelection, Level, Selection, Service,
    ServiceCategory, ServiceInfo,
};

/// Prefix of every Google scope URL in the catalogue.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::scopes::SCOPE_PREFIX;
/// assert_eq!(format!("{SCOPE_PREFIX}drive"), "https://www.googleapis.com/auth/drive");
/// ```
pub const SCOPE_PREFIX: &str = "https://www.googleapis.com/auth/";

/// Default selection: Read for the common Workspace services; optional services and
/// administrative capabilities are off by default.
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

/// Whether the catalogue can safely request a Write level for this service.
fn write_supported(service: Service) -> bool {
    !matches!(
        service,
        Service::AdminReports
            | Service::AdminDirectory
            | Service::AdminGroups
            | Service::CloudIdentity
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
        .filter(|level| level.level == Level::Read || write_supported(level.service))
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
        (Service::Docs, Level::Read) => &["documents.readonly"],
        (Service::Docs, Level::Write) => &["documents"],
        (Service::Sheets, Level::Read) => &["spreadsheets.readonly"],
        (Service::Sheets, Level::Write) => &["spreadsheets"],
        (Service::Slides, Level::Read) => &["presentations.readonly"],
        (Service::Slides, Level::Write) => &["presentations"],
        (Service::Forms, Level::Read) => &["forms.body.readonly"],
        (Service::Forms, Level::Write) => &["forms.body"],
        (Service::Tasks, Level::Read) => &["tasks.readonly"],
        (Service::Tasks, Level::Write) => &["tasks"],
        (Service::People, Level::Read) => &["contacts.readonly"],
        (Service::People, Level::Write) => &["contacts"],
        (Service::AdminReports, Level::Read) => &["admin.reports.audit.readonly"],
        (Service::AdminReports, Level::Write) => &["admin.reports.audit.readonly"],
        (Service::AdminDirectory, Level::Read) => &["admin.directory.user.readonly"],
        (Service::AdminDirectory, Level::Write) => &["admin.directory.user.readonly"],
        (Service::AdminGroups, Level::Read) => &["admin.directory.group.readonly"],
        (Service::AdminGroups, Level::Write) => &["admin.directory.group.readonly"],
        (Service::CloudIdentity, Level::Read) => &["cloud-identity.groups.readonly"],
        (Service::CloudIdentity, Level::Write) => &["cloud-identity.groups.readonly"],
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

/// Full scope URLs for a business connector selection.
pub fn business_scopes_for(selection: &BusinessSelection) -> Vec<String> {
    business_normalize(selection)
        .iter()
        .flat_map(|access| business_level_scopes(access.connector, access.level))
        .map(|short| format!("{SCOPE_PREFIX}{short}"))
        .collect()
}

/// Normalises business access so Write always includes Read.
pub fn business_normalize(selection: &BusinessSelection) -> BusinessSelection {
    let mut normalized = selection.clone();
    for access in selection
        .iter()
        .filter(|access| access.level == Level::Write)
    {
        normalized.insert(access.connector, Level::Read);
    }
    normalized
}

/// The `--scopes` argument for business connector access.
pub fn business_scopes_arg(selection: &BusinessSelection) -> String {
    business_scopes_for(selection).join(",")
}

fn business_level_scopes(connector: BusinessConnector, level: Level) -> &'static [&'static str] {
    match (connector, level) {
        (BusinessConnector::Youtube, Level::Read) => &["youtube.readonly"],
        (BusinessConnector::Youtube, Level::Write) => &["youtube"],
        (BusinessConnector::GoogleAds, _) => &["adwords"],
        (BusinessConnector::Analytics, Level::Read) => &["analytics.readonly"],
        (BusinessConnector::Analytics, Level::Write) => &["analytics.edit"],
        (BusinessConnector::SearchConsole, Level::Read) => &["webmasters.readonly"],
        (BusinessConnector::SearchConsole, Level::Write) => &["webmasters"],
        (BusinessConnector::BusinessProfile, _) => &["business.manage"],
        (BusinessConnector::MerchantCenter, _) => &["content"],
    }
}

/// Converts granted OAuth scopes into business connector levels.
pub fn business_selection_from_scopes(granted: &[String]) -> BusinessSelection {
    let is_granted = |short: &str| {
        granted
            .iter()
            .any(|scope| scope.strip_prefix(SCOPE_PREFIX) == Some(short))
    };
    let mut selection = BusinessSelection::new();
    for connector in [
        BusinessConnector::Youtube,
        BusinessConnector::GoogleAds,
        BusinessConnector::Analytics,
        BusinessConnector::SearchConsole,
        BusinessConnector::BusinessProfile,
        BusinessConnector::MerchantCenter,
    ] {
        for level in [Level::Read, Level::Write] {
            if business_level_scopes(connector, level)
                .iter()
                .all(|short| is_granted(short))
            {
                selection.insert(connector, level);
            }
        }
    }
    selection
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
    let is_granted = |short: &str| {
        granted
            .iter()
            .any(|scope| scope.strip_prefix(SCOPE_PREFIX) == Some(short))
    };
    let mut selection = Selection::new();
    for service in Service::ALL {
        for level in [Level::Read, Level::Write] {
            if level == Level::Write && !write_supported(service) {
                continue;
            }
            if level_scopes(service, level)
                .iter()
                .all(|short| is_granted(short))
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
/// assert_eq!(scopes::catalogue().len(), 15);
/// ```
pub fn catalogue() -> Vec<ServiceInfo> {
    catalogue_for(&Selection::new())
}

/// Builds the catalogue using the scopes already granted to the connected account.
///
/// Administrative rows are enabled only when their read scope is already present.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::{scopes, Level, Selection, Service};
/// let granted = Selection::from_levels([(Service::AdminReports, Level::Read)]);
/// let catalogue = scopes::catalogue_for(&granted);
/// assert!(catalogue.iter().find(|info| info.service == Service::AdminReports).unwrap().enabled);
/// ```
pub fn catalogue_for(granted: &Selection) -> Vec<ServiceInfo> {
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
                Service::Docs => ("Docs", "Read: read documents", "Write: edit documents"),
                Service::Sheets => ("Sheets", "Read: read spreadsheets", "Write: edit spreadsheets"),
                Service::Slides => ("Slides", "Read: read presentations", "Write: edit presentations"),
                Service::Forms => ("Forms", "Read: read forms", "Write: edit forms"),
                Service::Tasks => ("Tasks", "Read: read task lists", "Write: create and edit tasks"),
                Service::People => ("People", "Read: read contacts and profiles", "Write: edit contacts"),
                Service::AdminReports => (
                    "Admin Reports",
                    "Read: audit and usage reports",
                    "Write: unavailable for this API",
                ),
                Service::AdminDirectory => (
                    "Admin Directory",
                    "Read: directory users",
                    "Write: unavailable for this API",
                ),
                Service::AdminGroups => (
                    "Admin Groups",
                    "Read: groups and memberships",
                    "Write: unavailable for this API",
                ),
                Service::CloudIdentity => (
                    "Cloud Identity",
                    "Read: organization identity groups",
                    "Write: unavailable for this API",
                ),
            };
            ServiceInfo {
                service,
                name,
                read,
                write,
                category: if matches!(
                    service,
                    Service::AdminReports
                        | Service::AdminDirectory
                        | Service::AdminGroups
                        | Service::CloudIdentity
                ) {
                    ServiceCategory::Admin
                } else {
                    ServiceCategory::Workspace
                },
                write_enabled: write_supported(service),
                enabled: !matches!(
                    service,
                    Service::AdminReports
                        | Service::AdminDirectory
                        | Service::AdminGroups
                        | Service::CloudIdentity
                ) || granted.contains(service, Level::Read),
                availability_reason: if matches!(
                    service,
                    Service::AdminReports
                        | Service::AdminDirectory
                        | Service::AdminGroups
                        | Service::CloudIdentity
                ) && !granted.contains(service, Level::Read) {
                    Some("Requires a Google Workspace administrator account or delegated admin permissions.")
                } else {
                    None
                },
            }
        })
        .collect()
}

/// Business APIs that use the shared Google account but need dedicated connectors.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::scopes;
/// assert_eq!(scopes::business_catalogue().len(), 6);
/// ```
pub fn business_catalogue() -> Vec<BusinessConnectorInfo> {
    business_catalogue_for(&BusinessSelection::new())
}

/// Builds the business connector catalogue with currently granted access.
pub fn business_catalogue_for(granted: &BusinessSelection) -> Vec<BusinessConnectorInfo> {
    [
        (
            BusinessConnector::Youtube,
            "YouTube",
            "Read: channels, videos, comments and analytics metadata",
            "Write: upload and manage videos",
        ),
        (
            BusinessConnector::GoogleAds,
            "Google Ads",
            "Read: campaigns, ads, metrics and accounts",
            "Write: create and manage campaigns",
        ),
        (
            BusinessConnector::Analytics,
            "Google Analytics",
            "Read: properties, reports and metrics",
            "Write: manage selected property configuration",
        ),
        (
            BusinessConnector::SearchConsole,
            "Search Console",
            "Read: sites, queries and performance data",
            "Write: manage site verification and settings",
        ),
        (
            BusinessConnector::BusinessProfile,
            "Business Profile",
            "Read: locations, reviews and performance",
            "Write: manage locations and replies",
        ),
        (
            BusinessConnector::MerchantCenter,
            "Merchant Center",
            "Read: products, accounts and diagnostics",
            "Write: manage products and feeds",
        ),
    ]
    .into_iter()
    .map(|(connector, name, read, write)| BusinessConnectorInfo {
        connector,
        name,
        read,
        write,
        enabled: connector == BusinessConnector::Youtube
            || granted.contains(connector, Level::Read),
        availability_reason: if connector == BusinessConnector::Youtube
            || granted.contains(connector, Level::Read)
        {
            None
        } else {
            Some("Connector not implemented yet; YouTube is available first.")
        },
    })
    .collect()
}
