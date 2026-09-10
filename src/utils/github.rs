use anyhow::{Context, Result};
use serde::Deserialize;

use crate::utils::retry::with_retry;

/// A repository found via GitHub search.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name: String,
    pub description: Option<String>,
    pub stars: u32,
    pub url: String,
}

/// GitHub API response for search.
#[derive(Debug, Deserialize)]
struct GitHubSearchResponse {
    items: Vec<GitHubRepo>,
    total_count: u64,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    name: String,
    description: Option<String>,
    stargazers_count: u32,
    html_url: String,
}

/// Search for inx packages via GitHub API.
/// Queries repos in the given org that match the query string.
pub fn search_packages(org: &str, query: &str, token: Option<&str>) -> Result<Vec<SearchResult>> {
    let url = format!(
        "https://api.github.com/search/repositories?q={query}+org:{org}&sort=stars&per_page=20",
        query = urlencoding::encode(query)
    );

    let token_owned = token.map(str::to_string);

    with_retry(|| {
        let client = reqwest::blocking::Client::new();
        let mut req = client
            .get(&url)
            .header("User-Agent", "inx-pm")
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(tok) = &token_owned {
            req = req.header("Authorization", format!("Bearer {tok}"));
        }

        let resp = req
            .send()
            .with_context(|| "Failed to connect to GitHub API")?;

        if resp.status().as_u16() == 403 {
            anyhow::bail!(
                "GitHub API rate limit exceeded. Set GITHUB_TOKEN to increase the limit."
            );
        }

        if !resp.status().is_success() {
            anyhow::bail!("GitHub API returned status {}", resp.status());
        }

        let body: GitHubSearchResponse = resp
            .json()
            .with_context(|| "Failed to parse GitHub API response")?;

        if body.total_count == 0 {
            return Ok(Vec::new());
        }

        let results = body
            .items
            .into_iter()
            .map(|repo| SearchResult {
                name: repo.name,
                description: repo.description,
                stars: repo.stargazers_count,
                url: repo.html_url,
            })
            .collect();

        Ok(results)
    })
}

/// Get the GITHUB_TOKEN from environment if set.
pub fn get_github_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.is_empty())
}
