use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use lexicon_rs::repo::layout::RepoLayout;
use lexicon_rs::spec::auth::Provider;

use crate::app::AuthAction;
use crate::output;

fn spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.magenta} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

pub fn run(action: AuthAction) -> miette::Result<()> {
    match action {
        AuthAction::Login { provider, port } => run_login(provider, port),
        AuthAction::Refresh { provider } => run_refresh(provider),
        AuthAction::Status => run_status(),
        AuthAction::SetKey { provider, key } => run_set_key(provider, key),
        AuthAction::Logout { provider } => run_logout(provider),
    }
}

fn pick_provider(prompt: &str) -> miette::Result<Provider> {
    let items = vec!["claude  (browser OAuth)", "openai  (browser OAuth)"];
    let selection = dialoguer::Select::new()
        .with_prompt(prompt)
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| miette::miette!("prompt cancelled: {e}"))?;

    match selection {
        0 => Ok(Provider::Claude),
        1 => Ok(Provider::OpenAi),
        _ => unreachable!(),
    }
}

fn run_login(provider: Option<Provider>, port: Option<u16>) -> miette::Result<()> {
    let provider = match provider {
        Some(p) => p,
        None => pick_provider("Which AI provider?")?,
    };

    let layout = RepoLayout::discover()?;
    let _creds = lexicon_rs::core::auth::login(&layout, provider, port)?;

    println!();
    output::success(&format!(
        "Credentials saved (.lexicon/auth/{}.json)",
        provider.as_str()
    ));
    println!();
    Ok(())
}

fn run_refresh(provider: Option<Provider>) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    let provider = match provider {
        Some(p) => p,
        None => {
            let results = lexicon_rs::core::auth::status(&layout)?;
            let authenticated: Vec<Provider> = results
                .into_iter()
                .filter_map(|(p, c)| c.map(|_| p))
                .collect();

            if authenticated.is_empty() {
                return Err(miette::miette!(
                    "no credentials stored — run: lexicon auth login"
                ));
            }

            if authenticated.len() == 1 {
                authenticated[0]
            } else {
                let items: Vec<String> =
                    authenticated.iter().map(|p| p.as_str().to_owned()).collect();
                let selection = dialoguer::Select::new()
                    .with_prompt("Which provider to refresh?")
                    .items(&items)
                    .default(0)
                    .interact()
                    .map_err(|e| miette::miette!("prompt cancelled: {e}"))?;
                authenticated[selection]
            }
        }
    };

    let pb = spinner();
    pb.set_message(format!("Refreshing {} token...", provider.as_str()));

    let _refreshed = lexicon_rs::core::auth::refresh(&layout, provider)?;

    pb.finish_with_message("✓ Token refreshed and saved.".to_string());
    println!();
    Ok(())
}

fn run_set_key(provider: Provider, key: String) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;
    lexicon_rs::core::auth::set_key(&layout, provider, key)?;
    output::success(&format!(
        "API key saved for {} (.lexicon/auth/{}.json)",
        provider.display_name(),
        provider.as_str()
    ));
    Ok(())
}

fn run_status() -> miette::Result<()> {
    let layout = RepoLayout::discover()?;
    let results = lexicon_rs::core::auth::status(&layout)?;

    println!();
    let any_stored = results.iter().any(|(_, c)| c.is_some());
    if !any_stored {
        println!("  No credentials stored.");
        println!("  Run: lexicon auth login");
    } else {
        println!("  Stored credentials:");
        for (provider, creds) in &results {
            if let Some(c) = creds {
                let note = if c.is_expired() {
                    if c.refresh_token.is_some() {
                        "(expired — run: lexicon auth refresh)"
                    } else {
                        "(expired — run: lexicon auth login)"
                    }
                } else if c.expires_at.is_some() {
                    "(OAuth token)"
                } else {
                    "(API key)"
                };

                let style = if c.is_expired() {
                    console::Style::new().red()
                } else {
                    console::Style::new().green()
                };

                println!(
                    "    {} {}  {}",
                    console::Style::new().green().bold().apply_to("✓"),
                    provider.as_str(),
                    style.apply_to(note),
                );
            }
        }
    }
    println!();
    Ok(())
}

fn run_logout(provider: Option<Provider>) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    let provider = match provider {
        Some(p) => p,
        None => {
            let results = lexicon_rs::core::auth::status(&layout)?;
            let authenticated: Vec<Provider> = results
                .into_iter()
                .filter_map(|(p, c)| c.map(|_| p))
                .collect();

            if authenticated.is_empty() {
                println!("  No credentials stored.");
                return Ok(());
            }

            if authenticated.len() == 1 {
                authenticated[0]
            } else {
                let items: Vec<String> =
                    authenticated.iter().map(|p| p.as_str().to_owned()).collect();
                let selection = dialoguer::Select::new()
                    .with_prompt("Which provider to log out?")
                    .items(&items)
                    .default(0)
                    .interact()
                    .map_err(|e| miette::miette!("prompt cancelled: {e}"))?;
                authenticated[selection]
            }
        }
    };

    lexicon_rs::core::auth::remove(&layout, provider)?;
    output::success(&format!("Logged out of {}.", provider.as_str()));
    Ok(())
}
