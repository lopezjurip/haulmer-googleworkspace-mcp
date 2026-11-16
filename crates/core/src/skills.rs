//! T08 — `Core::install_skills()`: install the pinned gws skills into the agents.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use crate::{Core, CoreError, InstalledSkills, KeyStorage, Selection, Service};

/// Tarball of the gws repository at the pinned tag; skills live under `skills/`.
///
/// # Example
///
/// ```
/// assert!(haulmer_connect_core::skills::SKILLS_TARBALL_URL.ends_with("v0.22.5.tar.gz"));
/// ```
pub const SKILLS_TARBALL_URL: &str =
    "https://github.com/googleworkspace/cli/archive/refs/tags/v0.22.5.tar.gz";

/// The skill every per-service skill reads first; always installed.
const SHARED_SKILL: &str = "gws-shared";

/// Agent folders (under home) whose `skills/` folder receives the skills, when they exist.
const AGENT_DIRS: [&str; 2] = [".claude", ".agents"];

/// Files of one skill folder: path relative to the folder -> content.
type SkillFiles = BTreeMap<PathBuf, Vec<u8>>;

impl Core {
    /// Installs `gws-shared` plus the skills of the granted services into
    /// `<home>/.claude/skills` and `<home>/.agents/skills` (when their parent exists).
    /// The granted services come from [`Core::status`]; see [`Core::install_skills_for`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// for installed in core.install_skills().unwrap() {
    ///     println!("{}: {:?}", installed.target.display(), installed.skills);
    /// }
    /// ```
    pub fn install_skills(&self) -> Result<Vec<InstalledSkills>, CoreError> {
        self.install_skills_for(&self.status()?.granted)
    }

    /// Installs `gws-shared` plus the skills of the services in `granted` into every
    /// existing agent folder, replacing earlier copies and removing the skills of services
    /// no longer granted. Downloads [`SKILLS_TARBALL_URL`] only if some agent folder exists.
    /// Returns the installed skill folders per target, sorted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Level, Paths, Selection, Service};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// let granted = Selection::from_levels([(Service::Gmail, Level::Read)]);
    /// let installed = core.install_skills_for(&granted).unwrap();
    /// // e.g. [/h/.claude/skills: gws-gmail, gws-gmail-read, ..., gws-shared]
    /// ```
    pub fn install_skills_for(
        &self,
        granted: &Selection,
    ) -> Result<Vec<InstalledSkills>, CoreError> {
        let targets: Vec<PathBuf> = AGENT_DIRS
            .iter()
            .map(|dir| self.paths.home.join(dir))
            .filter(|dir| dir.is_dir())
            .map(|dir| dir.join("skills"))
            .collect();
        if targets.is_empty() {
            return Ok(Vec::new());
        }
        let mut skills = self.download_skills()?;
        if let (Some(section), Some(shared)) =
            (self.environment_section()?, skills.get_mut(SHARED_SKILL))
        {
            if let Some(content) = shared.get_mut(Path::new("SKILL.md")) {
                content.extend_from_slice(section.as_bytes());
            }
        }
        let wanted: BTreeMap<&String, &SkillFiles> = skills
            .iter()
            .filter(|(name, _)| *name == SHARED_SKILL || granted_skill(granted, name))
            .collect();

        let mut installed = Vec::new();
        for target in targets {
            fs::create_dir_all(&target)?;
            remove_ungranted(&target, granted)?;
            for (name, files) in &wanted {
                write_skill(&target.join(name), files)?;
            }
            installed.push(InstalledSkills {
                target,
                skills: wanted.keys().map(|name| name.to_string()).collect(),
            });
        }
        Ok(installed)
    }

    /// The "Environment" section for `gws-shared/SKILL.md` when the config folder or the
    /// key storage is not the gws default; `None` otherwise.
    fn environment_section(&self) -> Result<Option<String>, CoreError> {
        let settings = self.settings()?;
        let backend = match settings.resolved_key_storage() {
            KeyStorage::File => "file",
            _ => "keyring",
        };
        if settings.is_default_config_dir() && backend == "keyring" {
            return Ok(None);
        }
        Ok(Some(format!(
            "\n## Environment\n\n\
             haulmer-connect keeps the gws credentials outside the default folder. \
             Always run gws with these variables set:\n\n\
             ```bash\n\
             export GOOGLE_WORKSPACE_CLI_CONFIG_DIR=\"{}\"\n\
             export GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND={backend}\n\
             ```\n",
            settings.active_config_dir(&self.paths).display()
        )))
    }

    /// Downloads the pinned tarball and returns its skills by folder name.
    fn download_skills(&self) -> Result<BTreeMap<String, SkillFiles>, CoreError> {
        let response = self.http.get(SKILLS_TARBALL_URL)?;
        if response.status != 200 {
            return Err(CoreError::Download(format!(
                "skills download returned HTTP {}",
                response.status
            )));
        }
        extract_skills(&response.body)
    }
}

/// The service whose skills a folder name belongs to (`gws-gmail*`, `gws-meet`, ...).
fn skill_service(name: &str) -> Option<Service> {
    Service::ALL.into_iter().find(|service| {
        let base = format!("gws-{}", service_slug(*service));
        match service {
            Service::Meet => name == base,
            _ => name == base || name.starts_with(&format!("{base}-")),
        }
    })
}

/// The service's name in skill folders.
fn service_slug(service: Service) -> &'static str {
    match service {
        Service::Gmail => "gmail",
        Service::Calendar => "calendar",
        Service::Chat => "chat",
        Service::Meet => "meet",
        Service::Drive => "drive",
    }
}

/// Whether `name` is the skill of a service granted at any level.
fn granted_skill(granted: &Selection, name: &str) -> bool {
    skill_service(name).is_some_and(|service| granted.iter().any(|level| level.service == service))
}

/// Deletes service skill folders in `target` whose service is not granted.
fn remove_ungranted(target: &Path, granted: &Selection) -> Result<(), CoreError> {
    for entry in fs::read_dir(target)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if skill_service(&name).is_some() && !granted_skill(granted, &name) {
            fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}

/// Replaces the folder `dir` with exactly `files`.
fn write_skill(dir: &Path, files: &SkillFiles) -> Result<(), CoreError> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    for (relative, content) in files {
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
    }
    Ok(())
}

/// Reads `<root>/skills/<name>/<file>` entries of a `.tar.gz`. Rejects `..`, absolute paths
/// and links under `skills/`; ignores everything else.
fn extract_skills(archive: &[u8]) -> Result<BTreeMap<String, SkillFiles>, CoreError> {
    let unsafe_entry = |path: &Path| {
        CoreError::Download(format!("unsafe path in skills archive: {}", path.display()))
    };
    let mut skills: BTreeMap<String, SkillFiles> = BTreeMap::new();
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(archive));
    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        if !path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err(unsafe_entry(&path));
        }
        let parts: Vec<&std::ffi::OsStr> = path.iter().collect();
        if parts.len() < 4 || parts[1] != "skills" {
            continue;
        }
        let kind = entry.header().entry_type();
        if kind.is_symlink() || kind.is_hard_link() {
            return Err(unsafe_entry(&path));
        }
        if !kind.is_file() {
            continue;
        }
        let name = parts[2].to_string_lossy().into_owned();
        let relative: PathBuf = parts[3..].iter().collect();
        let mut content = Vec::new();
        entry.read_to_end(&mut content)?;
        skills.entry(name).or_default().insert(relative, content);
    }
    Ok(skills)
}
