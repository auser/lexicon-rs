use std::io::{Read, Write as IoWrite};
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::repo::layout::RepoLayout;
use crate::spec::auth::{Credentials, Provider, ProviderConfig};

use super::error::{CoreError, CoreResult};

const CALLBACK_TIMEOUT_SECS: u64 = 180;

/// Run the full PKCE OAuth login flow for the given provider.
pub fn login(
    layout: &RepoLayout,
    provider: Provider,
    port_override: Option<u16>,
) -> CoreResult<Credentials> {
    let config = provider.config();
    let port = port_override.unwrap_or(config.default_port);

    match pkce_oauth_flow(provider, &config, port) {
        Ok(creds) => {
            save(layout, &creds)?;
            Ok(creds)
        }
        Err(e) if port_override.is_some() && port != config.default_port => {
            eprintln!(
                "  ⚠ port {port} failed ({e}), retrying on default port {}...",
                config.default_port
            );
            let creds = pkce_oauth_flow(provider, &config, config.default_port)?;
            save(layout, &creds)?;
            Ok(creds)
        }
        Err(e) => Err(e),
    }
}

/// Refresh an expired token for the given provider.
pub fn refresh(layout: &RepoLayout, provider: Provider) -> CoreResult<Credentials> {
    let creds = load(layout, provider)?.ok_or_else(|| CoreError::NotAuthenticated {
        provider: provider.as_str().to_owned(),
    })?;

    let refresh_token = creds.refresh_token.as_deref().ok_or_else(|| {
        CoreError::RefreshFailed {
            provider: provider.as_str().to_owned(),
            reason: "no refresh_token stored — re-run: lexicon auth login".into(),
        }
    })?;

    let config = provider.config();
    let refreshed = exchange_refresh_token(&config, provider, refresh_token, &creds)?;
    save(layout, &refreshed)?;
    Ok(refreshed)
}

/// Load stored credentials for a provider.
pub fn load(layout: &RepoLayout, provider: Provider) -> CoreResult<Option<Credentials>> {
    let path = layout.auth_credential_path(provider.as_str());
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let creds: Credentials = serde_json::from_str(&text).map_err(|e| {
                CoreError::AuthFailed {
                    reason: format!("invalid credentials file: {e}"),
                }
            })?;
            Ok(Some(creds))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Save credentials to disk with restrictive file permissions.
pub fn save(layout: &RepoLayout, creds: &Credentials) -> CoreResult<()> {
    let dir = layout.auth_dir();
    std::fs::create_dir_all(&dir)?;

    let path = layout.auth_credential_path(creds.provider.as_str());
    let json = serde_json::to_string_pretty(creds).map_err(|e| CoreError::AuthFailed {
        reason: format!("serializing credentials: {e}"),
    })?;
    std::fs::write(&path, &json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

/// Remove stored credentials for a provider.
pub fn remove(layout: &RepoLayout, provider: Provider) -> CoreResult<()> {
    let path = layout.auth_credential_path(provider.as_str());
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Check authentication status for all providers.
pub fn status(layout: &RepoLayout) -> CoreResult<Vec<(Provider, Option<Credentials>)>> {
    let mut results = Vec::new();
    for provider in Provider::ALL {
        let creds = load(layout, provider)?;
        results.push((provider, creds));
    }
    Ok(results)
}

/// Save an API key as stored credentials (no OAuth, no expiry).
pub fn set_key(layout: &RepoLayout, provider: Provider, api_key: String) -> CoreResult<()> {
    let creds = Credentials {
        provider,
        access_token: api_key,
        refresh_token: None,
        expires_at: None,
    };
    save(layout, &creds)
}

/// Ensure the user is authenticated for the given provider.
///
/// Checks (in order): environment variable, stored credentials (with
/// auto-refresh for expired OAuth tokens).
pub fn ensure_authenticated(
    layout: &RepoLayout,
    provider: Provider,
) -> CoreResult<Credentials> {
    // 1. Check environment variable first
    let env_name = provider.env_var();
    if let Ok(key) = std::env::var(env_name) {
        if !key.is_empty() {
            return Ok(Credentials {
                provider,
                access_token: key,
                refresh_token: None,
                expires_at: None,
            });
        }
    }

    // 2. Fall back to stored credentials
    let creds = load(layout, provider)?.ok_or_else(|| CoreError::NotAuthenticated {
        provider: provider.as_str().to_owned(),
    })?;

    if creds.is_expired() {
        if creds.refresh_token.is_some() {
            return refresh(layout, provider);
        }
        return Err(CoreError::NotAuthenticated {
            provider: provider.as_str().to_owned(),
        });
    }

    Ok(creds)
}

fn pkce_oauth_flow(
    provider: Provider,
    config: &ProviderConfig,
    port: u16,
) -> CoreResult<Credentials> {
    let (verifier, challenge) = pkce_pair();
    let state = random_state();
    let redirect_uri = format!("http://localhost:{port}/callback");
    let scope_enc = config.scopes.replace(' ', "%20");

    let full_auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state={}",
        config.auth_url,
        config.client_id,
        percent_encode(&redirect_uri),
        scope_enc,
        challenge,
        state,
    );

    println!();
    println!("  → {} — OAuth authorization", provider.display_name());
    println!("  Opening authorization page in your browser...");
    println!("  (waiting up to {CALLBACK_TIMEOUT_SECS}s for the callback)");
    println!();

    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).map_err(|e| {
        CoreError::OAuthCallback(format!(
            "could not bind port {port} — is another instance running? ({e})"
        ))
    })?;

    let (tx, rx) = mpsc::channel::<CoreResult<(String, String)>>();
    let state_clone = state.clone();
    std::thread::spawn(move || {
        tx.send(accept_callback(&listener, &state_clone)).ok();
    });

    if let Err(e) = open::that(&full_auth_url) {
        eprintln!("  ⚠ Could not open browser: {e}");
        println!("  Please open this URL manually:");
        println!("  {full_auth_url}");
    }

    let (code, _returned_state) =
        match rx.recv_timeout(Duration::from_secs(CALLBACK_TIMEOUT_SECS)) {
            Ok(Ok(pair)) => pair,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(CoreError::OAuthCallback(format!(
                    "OAuth timed out after {CALLBACK_TIMEOUT_SECS} seconds"
                )));
            }
        };

    println!("  Exchanging authorization code...");
    let creds = exchange_code(config, provider, &code, &verifier, &redirect_uri, &state)?;

    println!("  ✓ Logged in to {}.", provider.display_name());

    Ok(creds)
}

fn accept_callback(
    listener: &TcpListener,
    expected_state: &str,
) -> CoreResult<(String, String)> {
    // Loop to skip non-callback requests (favicon, preflight, browser extensions).
    loop {
        let (mut stream, _) = listener
            .accept()
            .map_err(|e| CoreError::OAuthCallback(format!("accepting callback: {e}")))?;

        let mut buf = vec![0u8; 4096];
        let n = stream
            .read(&mut buf)
            .map_err(|e| CoreError::OAuthCallback(format!("reading callback: {e}")))?;
        let request = String::from_utf8_lossy(&buf[..n]);

        let first_line = request.lines().next().unwrap_or("");
        let path = first_line.split_whitespace().nth(1).unwrap_or("");

        // Ignore requests that aren't to /callback (e.g. /favicon.ico)
        if !path.starts_with("/callback") {
            let response = "HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n";
            stream.write_all(response.as_bytes()).ok();
            continue;
        }

        let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");

        let code = query_param(query, "code");
        let state = query_param(query, "state");
        let error = query_param(query, "error");
        let error_desc = query_param(query, "error_description");
        let ok = code.is_some() && state.as_deref() == Some(expected_state);

        let html = if ok {
            "<html><body style='font-family:sans-serif;padding:2em'>\
             <h2>Authorization successful</h2>\
             <p>You can close this tab and return to the terminal.</p>\
             </body></html>"
        } else {
            "<html><body style='font-family:sans-serif;padding:2em'>\
             <h2>Authorization failed</h2>\
             <p>State mismatch or missing code. Please try again.</p>\
             </body></html>"
        };

        let status_line = if ok { "200 OK" } else { "400 Bad Request" };
        let response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n{html}"
        );
        stream.write_all(response.as_bytes()).ok();

        if ok {
            return Ok((code.unwrap(), state.unwrap()));
        } else if let Some(err) = error {
            let detail = error_desc.unwrap_or_default();
            return Err(CoreError::OAuthCallback(format!(
                "provider returned error: {err} — {detail}"
            )));
        } else {
            return Err(CoreError::OAuthCallback(format!(
                "state mismatch or missing code (got query: {query})"
            )));
        }
    }
}

fn exchange_code(
    config: &ProviderConfig,
    provider: Provider,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
    state: &str,
) -> CoreResult<Credentials> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| CoreError::Http(e.to_string()))?;

    let resp = if config.token_exchange_json {
        client
            .post(config.token_url)
            .json(&serde_json::json!({
                "grant_type":    "authorization_code",
                "client_id":     config.client_id,
                "code":          code,
                "redirect_uri":  redirect_uri,
                "code_verifier": verifier,
                "state":         state,
            }))
            .send()
            .map_err(|e| CoreError::Http(format!("token exchange request: {e}")))?
    } else {
        client
            .post(config.token_url)
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", config.client_id),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("code_verifier", verifier),
            ])
            .send()
            .map_err(|e| CoreError::Http(format!("token exchange request: {e}")))?
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(CoreError::AuthFailed {
            reason: format!("token exchange failed ({status}): {body}"),
        });
    }

    parse_token_response(resp, provider)
}

fn exchange_refresh_token(
    config: &ProviderConfig,
    provider: Provider,
    refresh_token: &str,
    old_creds: &Credentials,
) -> CoreResult<Credentials> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| CoreError::Http(e.to_string()))?;

    let resp = if config.token_exchange_json {
        client
            .post(config.token_url)
            .json(&serde_json::json!({
                "grant_type":    "refresh_token",
                "client_id":     config.client_id,
                "refresh_token": refresh_token,
            }))
            .send()
            .map_err(|e| CoreError::Http(format!("refresh token request: {e}")))?
    } else {
        client
            .post(config.token_url)
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", config.client_id),
                ("refresh_token", refresh_token),
            ])
            .send()
            .map_err(|e| CoreError::Http(format!("refresh token request: {e}")))?
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(CoreError::RefreshFailed {
            provider: provider.as_str().to_owned(),
            reason: format!("({status}): {body}\n  Re-run: lexicon auth login"),
        });
    }

    let json: serde_json::Value = resp
        .json()
        .map_err(|e| CoreError::AuthFailed {
            reason: format!("parsing refresh response: {e}"),
        })?;

    let access_token = json["access_token"]
        .as_str()
        .ok_or_else(|| CoreError::AuthFailed {
            reason: "missing access_token in refresh response".into(),
        })?
        .to_owned();

    let new_refresh = json["refresh_token"]
        .as_str()
        .map(ToOwned::to_owned)
        .or_else(|| old_creds.refresh_token.clone());

    let expires_at = json["expires_in"].as_u64().map(|secs| now_secs() + secs);

    Ok(Credentials {
        provider,
        access_token,
        refresh_token: new_refresh,
        expires_at,
    })
}

fn parse_token_response(
    resp: reqwest::blocking::Response,
    provider: Provider,
) -> CoreResult<Credentials> {
    let json: serde_json::Value = resp.json().map_err(|e| CoreError::AuthFailed {
        reason: format!("parsing token response: {e}"),
    })?;

    let access_token = json["access_token"]
        .as_str()
        .ok_or_else(|| CoreError::AuthFailed {
            reason: "missing access_token".into(),
        })?
        .to_owned();

    let refresh_token = json["refresh_token"].as_str().map(ToOwned::to_owned);
    let expires_at = json["expires_in"].as_u64().map(|secs| now_secs() + secs);

    Ok(Credentials {
        provider,
        access_token,
        refresh_token,
        expires_at,
    })
}

fn pkce_pair() -> (String, String) {
    let mut raw = [0u8; 32];
    fill_random(&mut raw);
    let verifier = base64url(&raw);
    let digest = Sha256::digest(verifier.as_bytes());
    let challenge = base64url(&digest);
    (verifier, challenge)
}

fn random_state() -> String {
    let mut raw = [0u8; 32];
    fill_random(&mut raw);
    base64url(&raw)
}

fn fill_random(buf: &mut [u8]) {
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let _ = std::io::Read::read_exact(&mut f, buf);
    } else {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        for (i, b) in buf.iter_mut().enumerate() {
            *b = ((t.wrapping_shr(i as u32 % 32)) ^ i as u32) as u8;
        }
    }
}

/// Base64url encoding without padding (RFC 4648 §5).
fn base64url(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(data.len() * 4 / 3 + 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &byte in data {
        buf = (buf << 8) | u32::from(byte);
        bits += 8;
        while bits >= 6 {
            bits -= 6;
            out.push(CHARS[((buf >> bits) & 0x3f) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(CHARS[((buf << (6 - bits)) & 0x3f) as usize] as char);
    }
    out
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                let _ = std::fmt::Write::write_fmt(&mut out, format_args!("%{b:02X}"));
            }
        }
    }
    out
}

fn query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        if k == key { Some(v.to_owned()) } else { None }
    })
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64url_empty() {
        assert_eq!(base64url(&[]), "");
    }

    #[test]
    fn test_base64url_known_value() {
        assert_eq!(base64url(b"Hello"), "SGVsbG8");
    }

    #[test]
    fn test_pkce_pair_lengths() {
        let (verifier, challenge) = pkce_pair();
        assert_eq!(verifier.len(), 43);
        assert_eq!(challenge.len(), 43);
        assert_ne!(verifier, challenge);
    }

    #[test]
    fn test_random_state_length() {
        let state = random_state();
        assert_eq!(state.len(), 43);
    }

    #[test]
    fn test_percent_encode() {
        assert_eq!(percent_encode("hello"), "hello");
        assert_eq!(
            percent_encode("http://localhost:54321/callback"),
            "http%3A%2F%2Flocalhost%3A54321%2Fcallback"
        );
    }

    #[test]
    fn test_query_param() {
        assert_eq!(query_param("code=abc&state=xyz", "code"), Some("abc".into()));
        assert_eq!(query_param("code=abc&state=xyz", "state"), Some("xyz".into()));
        assert_eq!(query_param("code=abc&state=xyz", "missing"), None);
    }

    #[test]
    fn test_save_load_remove() {
        let dir = tempfile::tempdir().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());

        let creds = Credentials {
            provider: Provider::Claude,
            access_token: "test-token".into(),
            refresh_token: Some("refresh-tok".into()),
            expires_at: Some(9999999999),
        };

        save(&layout, &creds).unwrap();
        let path = layout.auth_credential_path("claude");
        assert!(path.exists());

        let loaded = load(&layout, Provider::Claude).unwrap().unwrap();
        assert_eq!(loaded.access_token, "test-token");
        assert_eq!(loaded.refresh_token.as_deref(), Some("refresh-tok"));
        assert_eq!(loaded.expires_at, Some(9999999999));

        let none = load(&layout, Provider::OpenAi).unwrap();
        assert!(none.is_none());

        remove(&layout, Provider::Claude).unwrap();
        let gone = load(&layout, Provider::Claude).unwrap();
        assert!(gone.is_none());

        remove(&layout, Provider::OpenAi).unwrap();
    }

    #[test]
    fn test_status_empty() {
        let dir = tempfile::tempdir().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());

        let results = status(&layout).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|(_, c)| c.is_none()));
    }

    #[cfg(unix)]
    #[test]
    fn test_save_sets_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());

        let creds = Credentials {
            provider: Provider::Claude,
            access_token: "secret".into(),
            refresh_token: None,
            expires_at: None,
        };
        save(&layout, &creds).unwrap();

        let path = layout.auth_credential_path("claude");
        let perms = std::fs::metadata(&path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o600);
    }
}
