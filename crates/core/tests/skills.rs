//! T08 — installing the pinned gws skills into the agents.
//!
//! `Core::install_skills()` reads the granted levels from `Core::status()` (T05), so these
//! tests drive `Core::install_skills_for(&Selection)`, which it delegates to.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use haulmer_connect_core::skills::SKILLS_TARBALL_URL;
use haulmer_connect_core::testing::{FakeHttp, FakeReply, FakeRunner};
use haulmer_connect_core::{
    Core, CoreError, HttpResponse, InstalledSkills, Level, Paths, Selection, Service,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

/// Every skill folder of the fixture tarball (a subset of the real v0.22.5 one).
const FIXTURE_SKILLS: &[&str] = &[
    "gws-shared",
    "gws-gmail",
    "gws-gmail-send",
    "gws-calendar",
    "gws-calendar-agenda",
    "gws-chat",
    "gws-chat-send",
    "gws-meet",
    "gws-drive",
    "gws-drive-upload",
    "gws-people",
    "recipe-email-digest",
];

/// Builds a `.tar.gz` mimicking the GitHub tag tarball: `cli-705fb0ecac6f4249679958f6325b809b63fdde17/...`.
fn tarball(extra: impl FnOnce(&mut tar::Builder<Vec<u8>>)) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    append(
        &mut builder,
        "cli-705fb0ecac6f4249679958f6325b809b63fdde17/README.md",
        "# gws\n",
    );
    for name in FIXTURE_SKILLS {
        let content = format!("---\nname: {name}\n---\n\n# {name}\n");
        append(
            &mut builder,
            &format!("cli-705fb0ecac6f4249679958f6325b809b63fdde17/skills/{name}/SKILL.md"),
            &content,
        );
    }
    append(
        &mut builder,
        "cli-705fb0ecac6f4249679958f6325b809b63fdde17/skills/gws-gmail/references/flags.md",
        "flags\n",
    );
    extra(&mut builder);
    let tar = builder.into_inner().unwrap();
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    std::io::Write::write_all(&mut gz, &tar).unwrap();
    gz.finish().unwrap()
}

/// Appends a regular file.
fn append(builder: &mut tar::Builder<Vec<u8>>, path: &str, content: &str) {
    let mut header = tar::Header::new_gnu();
    header.set_size(content.len() as u64);
    header.set_mode(0o644);
    header.set_entry_type(tar::EntryType::Regular);
    builder
        .append_data(&mut header, path, content.as_bytes())
        .unwrap();
}

/// Appends a regular file with a raw (unchecked) name, e.g. one containing `..`.
fn append_raw(builder: &mut tar::Builder<Vec<u8>>, raw_path: &str, content: &str) {
    let mut header = tar::Header::new_old();
    header.as_old_mut().name[..raw_path.len()].copy_from_slice(raw_path.as_bytes());
    header.set_size(content.len() as u64);
    header.set_mode(0o644);
    header.set_entry_type(tar::EntryType::Regular);
    header.set_cksum();
    builder.append(&header, content.as_bytes()).unwrap();
}

/// A temp home with the given agent folders, and a core whose HTTP serves `archive`.
struct Fixture {
    home: TempDir,
    _app_data: TempDir,
    http: Arc<FakeHttp>,
    core: Core,
}

impl Fixture {
    /// Serves `archive` and trusts its SHA-256 (the fixture stands in for the pinned tarball).
    fn new(agent_dirs: &[&str], archive: Vec<u8>) -> Self {
        let sha256: String = Sha256::digest(&archive)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let mut fixture = Self::serving(agent_dirs, HttpResponse::ok(archive));
        fixture.core = fixture.core.with_skills_sha256(&sha256);
        fixture
    }

    /// Serves `response` and expects the real pinned checksum.
    fn serving(agent_dirs: &[&str], response: HttpResponse) -> Self {
        let home = TempDir::new().unwrap();
        let app_data = TempDir::new().unwrap();
        for dir in agent_dirs {
            fs::create_dir(home.path().join(dir)).unwrap();
        }
        let http = Arc::new(FakeHttp::new());
        http.on(SKILLS_TARBALL_URL, response);
        let paths = Paths {
            home: home.path().into(),
            app_data: app_data.path().into(),
        };
        let core = Core::new(paths, Arc::new(FakeRunner::new()), http.clone());
        Self {
            home,
            _app_data: app_data,
            http,
            core,
        }
    }

    fn skills_dir(&self, agent: &str) -> std::path::PathBuf {
        self.home.path().join(agent).join("skills")
    }
}

/// Sorted folder names inside `dir`.
fn folders(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn strings(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

#[test]
fn installs_shared_and_granted_service_skills_into_both_agents() {
    let fixture = Fixture::new(&[".claude", ".agents"], tarball(|_| {}));
    let granted =
        Selection::from_levels([(Service::Gmail, Level::Read), (Service::Meet, Level::Read)]);

    let installed = fixture.core.install_skills_for(&granted).unwrap();

    let expected = strings(&["gws-gmail", "gws-gmail-send", "gws-meet", "gws-shared"]);
    assert_eq!(
        installed,
        [
            InstalledSkills {
                target: fixture.skills_dir(".claude"),
                skills: expected.clone(),
                skipped: vec![],
            },
            InstalledSkills {
                target: fixture.skills_dir(".agents"),
                skills: expected.clone(),
                skipped: vec![],
            },
        ]
    );
    for agent in [".claude", ".agents"] {
        let dir = fixture.skills_dir(agent);
        assert_eq!(folders(&dir), expected);
        assert_eq!(
            fs::read_to_string(dir.join("gws-meet/SKILL.md")).unwrap(),
            "---\nname: gws-meet\n---\n\n# gws-meet\n"
        );
        assert_eq!(
            fs::read_to_string(dir.join("gws-gmail/references/flags.md")).unwrap(),
            "flags\n"
        );
    }
}

#[test]
fn adds_generic_p0_skills_when_the_pinned_archive_does_not_have_them() {
    let fixture = Fixture::new(&[".claude"], tarball(|_| {}));

    fixture
        .core
        .install_skills_for(&Selection::from_levels([
            (Service::Docs, Level::Read),
            (Service::Sheets, Level::Write),
        ]))
        .unwrap();

    let dir = fixture.skills_dir(".claude");
    assert!(dir.join("gws-docs/SKILL.md").is_file());
    assert!(dir.join("gws-sheets/SKILL.md").is_file());
    assert!(!dir.join("gws-slides").exists());
}

#[test]
fn removes_skills_it_installed_for_services_no_longer_granted_and_keeps_others() {
    let fixture = Fixture::new(&[".claude"], tarball(|_| {}));
    let dir = fixture.skills_dir(".claude");
    fs::create_dir_all(dir.join("my-own-skill")).unwrap();
    fixture
        .core
        .install_skills_for(&Selection::from_levels([
            (Service::Gmail, Level::Write),
            (Service::Drive, Level::Read),
        ]))
        .unwrap();

    fixture
        .core
        .install_skills_for(&Selection::from_levels([(Service::Drive, Level::Read)]))
        .unwrap();

    assert_eq!(
        folders(&dir),
        strings(&[
            "gws-drive",
            "gws-drive-upload",
            "gws-shared",
            "my-own-skill"
        ])
    );
}

#[test]
fn custom_config_folder_adds_an_environment_section_to_the_shared_skill() {
    let fixture = Fixture::new(&[".claude", ".agents"], tarball(|_| {}));
    let custom = fixture.home.path().join("haulmer-gws");
    fixture.core.set_config_dir(Some(custom.clone())).unwrap();

    fixture.core.install_skills_for(&Selection::new()).unwrap();

    for agent in [".claude", ".agents"] {
        let shared =
            fs::read_to_string(fixture.skills_dir(agent).join("gws-shared/SKILL.md")).unwrap();
        assert!(shared.starts_with("---\nname: gws-shared\n---\n\n# gws-shared\n"));
        assert!(shared.contains("## Environment"), "{shared}");
        assert!(shared.contains(&format!(
            "export GOOGLE_WORKSPACE_CLI_CONFIG_DIR='{}'\nexport GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND=file\n",
            custom.display()
        )), "{shared}");
    }
}

#[test]
fn a_downloaded_gws_not_on_path_adds_its_folder_to_path_in_the_shared_skill() {
    let home = TempDir::new().unwrap();
    let app_data = TempDir::new().unwrap();
    fs::create_dir(home.path().join(".claude")).unwrap();
    let bin = app_data.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let gws = bin.join(if cfg!(windows) { "gws.exe" } else { "gws" });
    fs::write(&gws, "").unwrap();
    let runner = Arc::new(FakeRunner::new());
    // Only the downloaded copy answers; bare `gws` (PATH) has no rule and so is not found.
    runner.on_program(gws.clone(), &["--version"], FakeReply::ok("gws 0.22.5\n"));
    let archive = tarball(|_| {});
    let sha256: String = Sha256::digest(&archive)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let http = Arc::new(FakeHttp::new());
    http.on(SKILLS_TARBALL_URL, HttpResponse::ok(archive));
    let paths = Paths {
        home: home.path().into(),
        app_data: app_data.path().into(),
    };
    let core = Core::new(paths, runner, http).with_skills_sha256(&sha256);

    core.install_skills_for(&Selection::new()).unwrap();

    let shared =
        fs::read_to_string(home.path().join(".claude/skills/gws-shared/SKILL.md")).unwrap();
    assert!(
        shared.contains(&format!("export PATH='{}':\"$PATH\"\n", bin.display())),
        "{shared}"
    );
    assert!(
        !shared.contains("GOOGLE_WORKSPACE_CLI_CONFIG_DIR"),
        "{shared}"
    );
}

#[test]
fn default_config_folder_leaves_the_shared_skill_untouched() {
    let fixture = Fixture::new(&[".claude"], tarball(|_| {}));

    fixture.core.install_skills_for(&Selection::new()).unwrap();

    assert_eq!(
        fs::read_to_string(fixture.skills_dir(".claude").join("gws-shared/SKILL.md")).unwrap(),
        "---\nname: gws-shared\n---\n\n# gws-shared\n"
    );
}

#[test]
fn without_agent_folders_nothing_is_downloaded_or_written() {
    let fixture = Fixture::new(&[], tarball(|_| {}));

    let installed = fixture
        .core
        .install_skills_for(&Selection::from_levels([(Service::Gmail, Level::Read)]))
        .unwrap();

    assert!(installed.is_empty());
    assert!(fixture.http.requests().is_empty());
    assert!(!fixture.home.path().join(".claude").exists());
}

#[test]
fn a_failed_download_is_an_error() {
    let fixture = Fixture::serving(&[".claude"], HttpResponse::new(404, "Not Found"));

    let error = fixture
        .core
        .install_skills_for(&Selection::new())
        .unwrap_err();

    assert!(matches!(error, CoreError::Download(_)), "{error:?}");
}

/// Installing from `archive` fails and writes nothing.
fn assert_rejected(archive: Vec<u8>) {
    let fixture = Fixture::new(&[".claude"], archive);

    let error = fixture
        .core
        .install_skills_for(&Selection::new())
        .unwrap_err();

    assert!(matches!(error, CoreError::Download(_)), "{error:?}");
    assert!(!fixture.skills_dir(".claude").join("gws-shared").exists());
    assert!(!fixture.home.path().join("evil").exists());
}

#[test]
fn archive_entries_climbing_out_with_dot_dot_are_rejected() {
    assert_rejected(tarball(|builder| {
        append_raw(
            builder,
            "cli-705fb0ecac6f4249679958f6325b809b63fdde17/skills/../../evil/SKILL.md",
            "x",
        );
    }));
}

#[test]
fn archive_entries_with_absolute_paths_are_rejected() {
    assert_rejected(tarball(|builder| {
        append_raw(builder, "/tmp/evil/SKILL.md", "x");
    }));
}

#[test]
fn symlinks_inside_skills_are_rejected() {
    assert_rejected(tarball(|builder| {
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        builder
            .append_link(
                &mut header,
                "cli-705fb0ecac6f4249679958f6325b809b63fdde17/skills/gws-meet/evil",
                "/etc/passwd",
            )
            .unwrap();
    }));
}

#[test]
fn the_config_folder_is_single_quoted_for_posix_shells() {
    let fixture = Fixture::new(&[".claude"], tarball(|_| {}));
    let custom = fixture.home.path().join("it's $HOME");
    fixture.core.set_config_dir(Some(custom)).unwrap();

    fixture.core.install_skills_for(&Selection::new()).unwrap();

    let shared =
        fs::read_to_string(fixture.skills_dir(".claude").join("gws-shared/SKILL.md")).unwrap();
    let expected = format!(
        "export GOOGLE_WORKSPACE_CLI_CONFIG_DIR='{}/it'\\''s $HOME'\n",
        fixture.home.path().display()
    );
    assert!(shared.contains(&expected), "{shared}");
}

#[test]
fn a_tarball_not_matching_the_pinned_checksum_is_rejected_and_nothing_is_written() {
    let fixture = Fixture::serving(&[".claude"], HttpResponse::ok(tarball(|_| {})));

    let error = fixture
        .core
        .install_skills_for(&Selection::new())
        .unwrap_err();

    assert!(
        matches!(error, CoreError::ChecksumMismatch { .. }),
        "{error:?}"
    );
    assert!(!fixture.skills_dir(".claude").exists());
}

#[test]
fn skill_folders_it_did_not_install_are_skipped_untouched_and_never_removed() {
    let fixture = Fixture::new(&[".claude"], tarball(|_| {}));
    let dir = fixture.skills_dir(".claude");
    fs::create_dir_all(dir.join("gws-gmail")).unwrap();
    fs::write(dir.join("gws-gmail/SKILL.md"), "mine\n").unwrap();

    let installed = fixture
        .core
        .install_skills_for(&Selection::from_levels([(Service::Gmail, Level::Read)]))
        .unwrap();

    assert_eq!(
        installed[0].skills,
        strings(&["gws-gmail-send", "gws-shared"])
    );
    assert_eq!(installed[0].skipped, strings(&["gws-gmail"]));
    assert_eq!(
        fs::read_to_string(dir.join("gws-gmail/SKILL.md")).unwrap(),
        "mine\n"
    );

    fixture.core.install_skills_for(&Selection::new()).unwrap();

    assert_eq!(folders(&dir), strings(&["gws-gmail", "gws-shared"]));
    assert_eq!(
        fs::read_to_string(dir.join("gws-gmail/SKILL.md")).unwrap(),
        "mine\n"
    );
}
