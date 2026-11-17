//! T08 — `Core::install_skills()`: install the pinned gws skills into the agents.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use crate::gws_binary::sha256_hex;
use crate::ports::download;
use crate::{Core, CoreError, InstalledSkills, Selection, Service};

/// Commit of gws tag `v0.22.5` (a lightweight tag). Pinned by commit because tags can move.
///
/// # Example
///
/// ```
/// assert_eq!(haulmer_connect_core::skills::SKILLS_COMMIT.len(), 40);
/// ```
pub const SKILLS_COMMIT: &str = "705fb0ecac6f4249679958f6325b809b63fdde17";

/// Tarball of the gws repository at [`SKILLS_COMMIT`]; skills live under `<top>/skills/`.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::skills::{SKILLS_COMMIT, SKILLS_TARBALL_URL};
/// assert!(SKILLS_TARBALL_URL.ends_with(&format!("{SKILLS_COMMIT}.tar.gz")));
/// ```
pub const SKILLS_TARBALL_URL: &str = "https://github.com/googleworkspace/cli/archive/705fb0ecac6f4249679958f6325b809b63fdde17.tar.gz";

/// SHA-256 of [`SKILLS_TARBALL_URL`], computed when it was pinned. A download that does
/// not match is rejected before anything is extracted or written.
///
/// # Example
///
/// ```
/// assert_eq!(haulmer_connect_core::skills::SKILLS_TARBALL_SHA256.len(), 64);
/// ```
pub const SKILLS_TARBALL_SHA256: &str =
    "38a27bae67755c101da1278154ca4b9bf9f7fdccd7a5a1db916c2ff880bfa9fd";

/// File in `Paths::app_data` listing, per skills folder, the skills haulmer-connect installed.
/// Only those are ever replaced or removed.
const MANIFEST_FILE: &str = "installed-skills.json";

/// Skills folder path -> skill folder names haulmer-connect installed there.
type Manifest = BTreeMap<String, BTreeSet<String>>;

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
        let shared_skill_md = skills
            .get_mut(SHARED_SKILL)
            .and_then(|shared| shared.get_mut(Path::new("SKILL.md")));
        if let (Some(section), Some(content)) = (self.environment_section()?, shared_skill_md) {
            content.extend_from_slice(section.as_bytes());
        }
        let wanted: BTreeMap<&String, &SkillFiles> = skills
            .iter()
            .filter(|(name, _)| *name == SHARED_SKILL || granted_skill(granted, name))
            .collect();

        let mut manifest = self.read_manifest()?;
        let mut installed = Vec::new();
        for target in targets {
            fs::create_dir_all(&target)?;
            let key = target.to_string_lossy().into_owned();
            let owned = manifest.entry(key).or_default();
            // Remove what the app installed earlier and no longer wants.
            for name in owned.clone() {
                if !wanted.contains_key(&name) {
                    remove_dir_if_present(&target.join(&name))?;
                    owned.remove(&name);
                }
            }
            let (mut skills, mut skipped) = (Vec::new(), Vec::new());
            for (name, files) in &wanted {
                let dir = target.join(name);
                if dir.exists() && !owned.contains(*name) {
                    skipped.push(name.to_string());
                } else {
                    write_skill(&dir, files)?;
                    owned.insert(name.to_string());
                    skills.push(name.to_string());
                }
            }
            self.write_manifest(&manifest)?;
            installed.push(InstalledSkills {
                target,
                skills,
                skipped,
            });
        }
        Ok(installed)
    }

    /// Reads [`MANIFEST_FILE`]; empty when it does not exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let manifest = self.read_manifest()?;
    /// let owned = manifest.get("/home/ana/.claude/skills"); // Some({"gws-shared", ...})
    /// ```
    fn read_manifest(&self) -> Result<Manifest, CoreError> {
        let path = self.paths.app_data.join(MANIFEST_FILE);
        match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|error| CoreError::Parse(format!("{}: {error}", path.display()))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Manifest::new()),
            Err(error) => Err(error.into()),
        }
    }

    /// Writes [`MANIFEST_FILE`], creating `app_data` if missing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut manifest = self.read_manifest()?;
    /// manifest.entry("/home/ana/.claude/skills".into()).or_default().insert("gws-shared".into());
    /// self.write_manifest(&manifest)?;
    /// ```
    fn write_manifest(&self, manifest: &Manifest) -> Result<(), CoreError> {
        fs::create_dir_all(&self.paths.app_data)?;
        let json = serde_json::to_vec_pretty(manifest)?;
        fs::write(self.paths.app_data.join(MANIFEST_FILE), json)?;
        Ok(())
    }

    /// The "Environment" section for `gws-shared/SKILL.md` when agents need extra setup:
    /// the gws binary the app uses is not on PATH, the config folder is custom, or the key
    /// storage is not keyring. `None` when gws defaults work as-is.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // custom folder + downloaded gws:
    /// // export PATH='<app_data>/bin':"$PATH"
    /// // export GOOGLE_WORKSPACE_CLI_CONFIG_DIR='<dir>'
    /// // export GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND=file
    /// let section = self.environment_section()?;
    /// ```
    fn environment_section(&self) -> Result<Option<String>, CoreError> {
        let settings = self.settings()?;
        let backend = settings.resolved_key_storage().backend();
        let mut lines = Vec::new();
        // `find_gws` returns the bare name "gws" only when it is found on PATH.
        if let Some(gws) = self.find_gws()?
            && gws != Path::new("gws")
            && let Some(dir) = gws.parent()
        {
            lines.push(format!(
                "export PATH={}:\"$PATH\"",
                shell_quote(&dir.to_string_lossy())
            ));
        }
        if !settings.is_default_config_dir() {
            lines.push(format!(
                "export GOOGLE_WORKSPACE_CLI_CONFIG_DIR={}",
                shell_quote(&settings.active_config_dir(&self.paths).to_string_lossy())
            ));
        }
        if !settings.is_default_config_dir() || backend != "keyring" {
            lines.push(format!(
                "export GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND={backend}"
            ));
        }
        if lines.is_empty() {
            return Ok(None);
        }
        Ok(Some(format!(
            "\n## Environment\n\n\
             haulmer-connect set up gws for this machine. \
             Always run gws with these variables set:\n\n\
             ```bash\n{}\n```\n",
            lines.join("\n")
        )))
    }

    /// Downloads the pinned tarball, checks its SHA-256 (a mismatch is
    /// [`CoreError::ChecksumMismatch`] before anything is extracted) and returns its skills
    /// by folder name.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let skills = self.download_skills()?;
    /// assert!(skills.contains_key("gws-shared"));
    /// ```
    fn download_skills(&self) -> Result<BTreeMap<String, SkillFiles>, CoreError> {
        let archive = download(self.http.as_ref(), SKILLS_TARBALL_URL)?;
        let actual = sha256_hex(&archive);
        if actual != self.skills_sha256 {
            return Err(CoreError::ChecksumMismatch {
                expected: self.skills_sha256.clone(),
                actual,
            });
        }
        let mut skills = extract_skills(&archive)?;
        add_builtin_workspace_skills(&mut skills);
        Ok(skills)
    }
}

/// Adds stable generic skills for Workspace APIs that may not exist in the pinned
/// gws skill tarball. The CLI still discovers their live methods with `help`/`schema`.
fn add_builtin_workspace_skills(skills: &mut BTreeMap<String, SkillFiles>) {
    for service in [
        Service::Docs,
        Service::Sheets,
        Service::Slides,
        Service::Forms,
        Service::Tasks,
    ] {
        let name = format!("gws-{}", service.slug());
        skills.entry(name.clone()).or_insert_with(|| {
            let content = format!(
                "---\nname: {name}\ndescription: Google Workspace {} API.\n---\n\n\
                 # {name}\n\
                 Read `../gws-shared/SKILL.md` first.\n\n\
                 Discover available resources and methods with:\n\
                 ```bash\n\
                 gws {} --help\n\
                 gws schema {}.<resource>.<method>\n\
                 ```\n\n\
                 Always request JSON output and inspect the schema before write operations.\n",
                service.slug(),
                service.slug(),
                service.slug(),
            );
            BTreeMap::from([(PathBuf::from("SKILL.md"), content.into_bytes())])
        });
    }
}

/// The service whose skills a folder name belongs to (`gws-gmail*`, `gws-meet`, ...).
fn skill_service(name: &str) -> Option<Service> {
    Service::ALL.into_iter().find(|service| {
        let base = format!("gws-{}", service.slug());
        match service {
            Service::Meet => name == base,
            _ => name == base || name.starts_with(&format!("{base}-")),
        }
    })
}

/// Whether `name` is the skill of a service granted at any level.
fn granted_skill(granted: &Selection, name: &str) -> bool {
    skill_service(name).is_some_and(|service| granted.iter().any(|level| level.service == service))
}

/// Deletes the folder `dir` if it exists.
///
/// # Example
///
/// ```ignore
/// remove_dir_if_present(&target.join("gws-gmail"))?;
/// ```
fn remove_dir_if_present(dir: &Path) -> Result<(), CoreError> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}

/// Quotes `text` as one POSIX shell word: single quotes, embedded `'` written as `'\''`.
///
/// # Example
///
/// ```ignore
/// assert_eq!(shell_quote("/Users/ana/it's $HOME"), r"'/Users/ana/it'\''s $HOME'");
/// ```
fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

/// Replaces the folder `dir` with exactly `files`.
///
/// # Example
///
/// ```ignore
/// write_skill(&target.join("gws-shared"), &skills["gws-shared"])?;
/// ```
fn write_skill(dir: &Path, files: &SkillFiles) -> Result<(), CoreError> {
    remove_dir_if_present(dir)?;
    for (relative, content) in files {
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
    }
    Ok(())
}

/// Reads `<top>/skills/<name>/<file>` entries of a `.tar.gz`, whatever the top folder is
/// called (`cli-<commit>/` for a commit tarball): the first path component is stripped.
/// Rejects `..`, absolute paths and links under `skills/`; ignores everything else.
///
/// # Example
///
/// ```ignore
/// let skills = extract_skills(&archive)?; // "gws-shared" -> {"SKILL.md": b"..."}
/// ```
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
