use std::sync::Arc;

use haulmer_connect_core::gws_env::write_marker;
use haulmer_connect_core::testing::{FakeHttp, FakeReply, FakeRunner, HttpRequest};
use haulmer_connect_core::{Core, HttpResponse, Paths};
use tempfile::TempDir;

const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const YOUTUBE_URL: &str = "https://www.googleapis.com/youtube/v3/channels?part=snippet,contentDetails,statistics&mine=true";

struct Fixture {
    _home: TempDir,
    _app_data: TempDir,
    runner: Arc<FakeRunner>,
    http: Arc<FakeHttp>,
    core: Core,
}

fn fixture() -> Fixture {
    let home = TempDir::new().unwrap();
    let app_data = TempDir::new().unwrap();
    let runner = Arc::new(FakeRunner::new());
    let http = Arc::new(FakeHttp::new());
    let paths = Paths {
        home: home.path().into(),
        app_data: app_data.path().into(),
    };
    let core = Core::new(paths, runner.clone(), http.clone());
    let config = home.path().join(".config/gws");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("credentials.enc"), b"encrypted").unwrap();
    write_marker(&config).unwrap();
    runner.on_program("gws", &["--version"], FakeReply::ok("gws 0.22.5\n"));
    Fixture {
        _home: home,
        _app_data: app_data,
        runner,
        http,
        core,
    }
}

#[test]
fn youtube_channel_refreshes_gws_credentials_and_uses_bearer_auth() {
    let fixture = fixture();
    fixture.runner.on(
        &["auth", "export", "--unmasked"],
        FakeReply::ok(r#"{"refresh_token":"refresh-token"}"#),
    );
    fixture.http.on(
        TOKEN_URL,
        HttpResponse::ok(r#"{"access_token":"access-token","expires_in":3600}"#),
    );
    fixture.http.on(
        YOUTUBE_URL,
        HttpResponse::ok(r#"{"items":[{"snippet":{"title":"Haulmer"}}]}"#),
    );

    let channel = fixture.core.youtube_channel().unwrap();

    assert_eq!(channel["items"][0]["snippet"]["title"], "Haulmer");
    assert!(fixture.http.requests().iter().any(|request| matches!(
        request,
        HttpRequest::PostForm { url, form }
            if url == TOKEN_URL
                && form.iter().any(|(key, value)| key == "refresh_token" && value == "refresh-token")
    )));
    assert!(fixture.http.requests().iter().any(|request| matches!(
        request,
        HttpRequest::GetBearer { url, token }
            if url == YOUTUBE_URL && token == "access-token"
    )));
}
