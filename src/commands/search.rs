use anyhow::Result;

use crate::utils::colors::{Colors, ICON_SEARCH};
use crate::utils::github::{get_github_token, search_packages};

/// Run `inx search <query>`.
pub fn run(query: &str, org: &str) -> Result<()> {
    let token = get_github_token();
    let results = search_packages(org, query, token.as_deref())?;

    if results.is_empty() {
        println!(
            "{} No packages found for '{}'",
            ICON_SEARCH,
            Colors::accent(query)
        );
        return Ok(());
    }

    println!(
        "{} {} for '{}':",
        ICON_SEARCH,
        Colors::bold(&format!("{} package(s) found", results.len())),
        Colors::accent(query)
    );
    println!();

    for repo in &results {
        let desc = repo
            .description
            .as_deref()
            .unwrap_or("No description");

        println!(
            "  {} {} {} {}",
            Colors::accent(&repo.name),
            Colors::dim(desc),
            Colors::dim("★"),
            Colors::dim(&repo.stars.to_string()),
        );
        println!(
            "    {}",
            Colors::dim(&repo.url)
        );
    }

    println!();
    println!(
        "{} {}",
        Colors::dim("Install with:"),
        Colors::bold("inx install <package-name>")
    );

    Ok(())
}
