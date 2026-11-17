//! T02: scope catalogue. Expected values come from the spec table (`docs/spec.md`, "Scope catalogue").

use haulmer_connect_core::scopes;
use haulmer_connect_core::{Level, Selection, Service};

#[test]
fn default_selection_is_read_for_gmail_calendar_chat_meet_only() {
    assert_eq!(
        scopes::default_selection(),
        Selection::from_levels([
            (Service::Gmail, Level::Read),
            (Service::Calendar, Level::Read),
            (Service::Chat, Level::Read),
            (Service::Meet, Level::Read),
        ])
    );
}

#[test]
fn normalize_adds_read_for_every_write_and_keeps_the_rest() {
    let selection = Selection::from_levels([
        (Service::Gmail, Level::Write),
        (Service::Drive, Level::Write),
        (Service::Chat, Level::Read),
    ]);
    assert_eq!(
        scopes::normalize(&selection),
        Selection::from_levels([
            (Service::Gmail, Level::Read),
            (Service::Gmail, Level::Write),
            (Service::Drive, Level::Read),
            (Service::Drive, Level::Write),
            (Service::Chat, Level::Read),
        ])
    );
}

/// Full URL for a short scope name, written out so expectations stay independent of the code.
fn url(short: &str) -> String {
    format!("https://www.googleapis.com/auth/{short}")
}

/// `scopes_for` as a sorted list, after checking it has no duplicates.
fn sorted_scopes(selection: &Selection) -> Vec<String> {
    let mut urls = scopes::scopes_for(selection);
    let count = urls.len();
    urls.sort();
    urls.dedup();
    assert_eq!(urls.len(), count, "duplicate scopes in {urls:?}");
    urls
}

fn sorted(shorts: &[&str]) -> Vec<String> {
    let mut urls: Vec<String> = shorts.iter().map(|short| url(short)).collect();
    urls.sort();
    urls
}

#[test]
fn every_read_row_of_the_spec_table_maps_to_exactly_its_scopes() {
    let rows: [(Service, &[&str]); 5] = [
        (Service::Gmail, &["gmail.readonly"]),
        (Service::Calendar, &["calendar.readonly"]),
        (
            Service::Chat,
            &[
                "chat.spaces.readonly",
                "chat.messages.readonly",
                "chat.memberships.readonly",
            ],
        ),
        (Service::Meet, &["meetings.space.readonly"]),
        (Service::Drive, &["drive.readonly"]),
    ];
    for (service, expected) in rows {
        let selection = Selection::from_levels([(service, Level::Read)]);
        assert_eq!(
            sorted_scopes(&selection),
            sorted(expected),
            "{service:?} Read"
        );
    }
}

#[test]
fn every_write_row_maps_to_its_write_scopes_plus_its_read_scopes() {
    let rows: [(Service, &[&str]); 5] = [
        (
            Service::Gmail,
            &["gmail.readonly", "gmail.send", "gmail.modify"],
        ),
        (Service::Calendar, &["calendar.readonly", "calendar.events"]),
        (
            Service::Chat,
            &[
                "chat.spaces.readonly",
                "chat.messages.readonly",
                "chat.memberships.readonly",
                "chat.messages",
            ],
        ),
        (
            Service::Meet,
            &["meetings.space.readonly", "meetings.space.created"],
        ),
        (Service::Drive, &["drive.readonly", "drive"]),
    ];
    for (service, expected) in rows {
        let selection = Selection::from_levels([(service, Level::Write)]);
        assert_eq!(
            sorted_scopes(&selection),
            sorted(expected),
            "{service:?} Write"
        );
    }
}

#[test]
fn empty_selection_has_no_scopes() {
    assert!(scopes::scopes_for(&Selection::new()).is_empty());
}

#[test]
fn scopes_arg_is_the_full_urls_joined_with_commas() {
    let selection = Selection::from_levels([(Service::Calendar, Level::Write)]);
    let arg = scopes::scopes_arg(&selection);
    let mut parts: Vec<&str> = arg.split(',').collect();
    parts.sort();
    assert_eq!(
        parts,
        [
            "https://www.googleapis.com/auth/calendar.events",
            "https://www.googleapis.com/auth/calendar.readonly",
        ]
    );
    assert!(!arg.contains(' '));
}

#[test]
fn reverse_mapping_round_trips_every_normalized_selection() {
    // Each service is off, Read, or Read + Write: 3^5 selections.
    for code in 0..3u32.pow(5) {
        let mut selection = Selection::new();
        for (index, service) in Service::ALL.into_iter().enumerate() {
            match code / 3u32.pow(index as u32) % 3 {
                1 => {
                    selection.insert(service, Level::Read);
                }
                2 => {
                    selection.insert(service, Level::Read);
                    selection.insert(service, Level::Write);
                }
                _ => {}
            }
        }
        let granted = scopes::scopes_for(&selection);
        assert_eq!(scopes::selection_from_scopes(&granted), selection);
    }
}

#[test]
fn a_level_is_granted_only_when_all_its_scopes_are() {
    // Person unticked chat.memberships.readonly and gmail.modify on the consent screen.
    let granted = [
        url("chat.spaces.readonly"),
        url("chat.messages.readonly"),
        url("gmail.readonly"),
        url("gmail.send"),
    ];
    assert_eq!(
        scopes::selection_from_scopes(&granted),
        Selection::from_levels([(Service::Gmail, Level::Read)])
    );
}

#[test]
fn identity_scopes_are_ignored() {
    let granted = [
        "openid".to_string(),
        "email".to_string(),
        "profile".to_string(),
        url("userinfo.email"),
        url("userinfo.profile"),
        url("drive.readonly"),
    ];
    assert_eq!(
        scopes::selection_from_scopes(&granted),
        Selection::from_levels([(Service::Drive, Level::Read)])
    );
    assert!(scopes::selection_from_scopes(&granted[..5]).is_empty());
}

#[test]
fn catalogue_lists_every_service_in_ui_order_with_plain_labels() {
    let catalogue = scopes::catalogue();
    let services: Vec<Service> = catalogue.iter().map(|info| info.service).collect();
    assert_eq!(
        services,
        [
            Service::Gmail,
            Service::Calendar,
            Service::Chat,
            Service::Meet,
            Service::Drive,
            Service::Docs,
            Service::Sheets,
            Service::Slides,
            Service::Forms,
            Service::Tasks,
            Service::People,
            Service::AdminReports,
            Service::AdminDirectory,
            Service::AdminGroups,
            Service::CloudIdentity,
        ]
    );
    let names: Vec<&str> = catalogue.iter().map(|info| info.name).collect();
    assert_eq!(
        names,
        [
            "Gmail",
            "Calendar",
            "Chat",
            "Meet",
            "Drive",
            "Docs",
            "Sheets",
            "Slides",
            "Forms",
            "Tasks",
            "People",
            "Admin Reports",
            "Admin Directory",
            "Admin Groups",
            "Cloud Identity",
        ]
    );
    assert_eq!(catalogue[0].read, "Read: read email");
    assert_eq!(catalogue[0].write, "Write: send and modify email");
    for info in &catalogue {
        assert!(
            info.read.len() > "Read: ".len() && info.read.starts_with("Read: "),
            "{info:?}"
        );
        assert!(
            info.write.len() > "Write: ".len() && info.write.starts_with("Write: "),
            "{info:?}"
        );
    }
    assert!(catalogue[..11].iter().all(|info| info.enabled));
    assert!(catalogue[11..].iter().all(|info| !info.enabled));
    assert!(catalogue[11..].iter().all(|info| !info.write_enabled));
}
