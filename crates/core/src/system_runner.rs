//! The real [`ProcessRunner`] on `std::process`.

use crate::{CommandSpec, CoreError, ProcessOutput, ProcessRunner, RunningProcess};

/// Runs programs with `std::process`: stdin closed, env and cwd from the [`CommandSpec`].
///
/// # Example
///
/// ```no_run
/// use haulmer_connect_core::{CommandSpec, ProcessRunner, SystemRunner};
/// let output = SystemRunner.run(&CommandSpec::new("gws", &["--version"], ".")).unwrap();
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemRunner;

impl ProcessRunner for SystemRunner {
    fn run(&self, command: &CommandSpec) -> Result<ProcessOutput, CoreError> {
        let _ = command;
        Err(CoreError::NotImplemented("SystemRunner::run"))
    }

    fn spawn(&self, command: &CommandSpec) -> Result<Box<dyn RunningProcess>, CoreError> {
        let _ = command;
        Err(CoreError::NotImplemented("SystemRunner::spawn"))
    }
}
