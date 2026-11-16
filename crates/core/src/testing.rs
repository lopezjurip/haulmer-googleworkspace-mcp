//! Test doubles for the ports, shared by every ticket's tests.

use crate::{
    CommandSpec, CoreError, HttpClient, HttpResponse, ProcessOutput, ProcessRunner, RunningProcess,
};

/// Fake [`ProcessRunner`].
#[derive(Default)]
pub struct FakeRunner;

impl FakeRunner {
    /// Creates a runner with no canned replies.
    pub fn new() -> Self {
        Self
    }
}

impl ProcessRunner for FakeRunner {
    fn run(&self, command: &CommandSpec) -> Result<ProcessOutput, CoreError> {
        let _ = command;
        Err(CoreError::NotImplemented("FakeRunner::run"))
    }

    fn spawn(&self, command: &CommandSpec) -> Result<Box<dyn RunningProcess>, CoreError> {
        let _ = command;
        Err(CoreError::NotImplemented("FakeRunner::spawn"))
    }
}

/// Fake [`HttpClient`].
#[derive(Default)]
pub struct FakeHttp;

impl FakeHttp {
    /// Creates a client with no canned responses.
    pub fn new() -> Self {
        Self
    }
}

impl HttpClient for FakeHttp {
    fn get(&self, url: &str) -> Result<HttpResponse, CoreError> {
        let _ = url;
        Err(CoreError::NotImplemented("FakeHttp::get"))
    }

    fn post_form(&self, url: &str, form: &[(&str, &str)]) -> Result<HttpResponse, CoreError> {
        let _ = (url, form);
        Err(CoreError::NotImplemented("FakeHttp::post_form"))
    }
}
