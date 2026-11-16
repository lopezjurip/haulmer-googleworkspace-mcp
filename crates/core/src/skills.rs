//! T08 — `Core::install_skills()`: install the pinned gws skills into the agents.

use crate::{Core, CoreError, InstalledSkills};

/// Tarball of the gws repository at the pinned tag; skills live under `skills/`.
///
/// # Example
///
/// ```
/// assert!(haulmer_connect_core::skills::SKILLS_TARBALL_URL.ends_with("v0.22.5.tar.gz"));
/// ```
pub const SKILLS_TARBALL_URL: &str =
    "https://github.com/googleworkspace/cli/archive/refs/tags/v0.22.5.tar.gz";

impl Core {
    /// Installs `gws-shared` plus the skills of the granted services into
    /// `<home>/.claude/skills` and `<home>/.agents/skills` (when their parent exists).
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
        Err(CoreError::NotImplemented("install_skills (T08)"))
    }
}
