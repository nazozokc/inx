use anyhow::{Context, Result};
use git2::Repository;
use std::fs;
use std::path::Path;

use crate::utils::config::{ensure_inx_dirs, get_registry_dir, load_config};
use crate::utils::constants::TAG_NAME_REGEX;

/// Initialize the registry: clone if missing, pull if already present.
pub fn init_registry() -> Result<()> {
    let registry_dir = get_registry_dir()?;
    let config = load_config()?;

    ensure_inx_dirs()?;

    let is_repo = Repository::open(&registry_dir).is_ok();

    if is_repo {
        println!("Updating registry...");
        pull(&registry_dir)?;
    } else {
        println!("Cloning registry...");
        let repo_url = if config.registry.ends_with(".git") {
            config.registry.clone()
        } else {
            format!("{}.git", config.registry)
        };
        let _ = fs::remove_dir_all(&registry_dir);
        git2::Repository::clone(&repo_url, &registry_dir)
            .with_context(|| format!("Failed to clone registry from {}", repo_url))?;
    }
    Ok(())
}

/// Update the registry with a git pull. Initializes if not present.
pub fn update_registry() -> Result<()> {
    let registry_dir = get_registry_dir()?;

    if !registry_dir.exists() || Repository::open(&registry_dir).is_err() {
        return init_registry();
    }

    println!("Pulling latest changes...");
    pull(&registry_dir)
}

/// Get all tags from the registry repository.
pub fn get_tags() -> Result<Vec<String>> {
    let registry_dir = get_registry_dir()?;
    let repo = Repository::open(&registry_dir)
        .with_context(|| format!("Failed to open repository: {}", registry_dir.display()))?;
    let tags = match repo.tag_names(None) {
        Ok(tags) => tags.iter().flatten().map(str::to_string).collect(),
        Err(_) => Vec::new(),
    };
    Ok(tags)
}

/// Validate a tag name. The regex itself allows `..`, so check it here too.
fn validate_tag_name(tag: &str) -> bool {
    TAG_NAME_REGEX.is_match(tag) && !tag.contains("..") && !tag.starts_with('-')
}

/// Checkout a specific tag in the registry repository.
pub fn checkout_tag(tag: &str) -> Result<()> {
    if !validate_tag_name(tag) {
        anyhow::bail!("Invalid tag name '{tag}'");
    }

    let registry_dir = get_registry_dir()?;
    let repo = Repository::open(&registry_dir)
        .with_context(|| format!("Failed to open repository: {}", registry_dir.display()))?;

    let tag_oid = repo
        .revparse_single(tag)
        .with_context(|| format!("Failed to resolve tag '{tag}'"))?
        .id();

    let mut object = repo
        .find_object(tag_oid, Some(git2::ObjectType::Commit))
        .with_context(|| format!("Failed to find object for tag '{tag}'"))?;

    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    repo.reset(&mut object, git2::ResetType::Hard, Some(&mut checkout_builder))
        .with_context(|| format!("Failed to checkout tag '{tag}'"))?;
    Ok(())
}

/// Fetch from origin and hard-reset to the upstream of the current branch.
/// Falls back to origin/main or origin/master when HEAD is detached
/// (e.g. after a tag checkout during `upgrade`).
fn pull(dir: &Path) -> Result<()> {
    let repo = Repository::open(dir)
        .with_context(|| format!("Failed to open repository: {}", dir.display()))?;

    let mut remote = repo
        .find_remote("origin")
        .context("Failed to find remote 'origin'")?;
    remote
        .fetch(
            &[
                "+refs/heads/main:refs/remotes/origin/main",
                "+refs/heads/master:refs/remotes/origin/master",
            ],
            None,
            None,
        )
        .context("Failed to fetch from origin")?;

    let head = repo.head().context("Failed to read HEAD")?;
    let target_oid = if head.is_branch() {
        let branch = head.shorthand().unwrap_or("main").to_string();
        match resolve_remote_branch(&repo, &branch) {
            Ok(oid) => oid,
            Err(_) => resolve_remote_branch(&repo, "main")
                .or_else(|_| resolve_remote_branch(&repo, "master"))
                .context("Failed to resolve upstream of current branch")?,
        }
    } else {
        // Detached HEAD: reset to the default branch.
        resolve_remote_branch(&repo, "main")
            .or_else(|_| resolve_remote_branch(&repo, "master"))
            .context("Failed to resolve origin/main or origin/master")?
    };

    let mut object = repo
        .find_object(target_oid, Some(git2::ObjectType::Commit))
        .context("Failed to find fetched commit")?;
    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    repo.reset(&mut object, git2::ResetType::Hard, Some(&mut checkout_builder))
        .context("Failed to reset to fetched commit")?;
    Ok(())
}

fn resolve_remote_branch(repo: &Repository, branch: &str) -> Result<git2::Oid> {
    repo.refname_to_id(&format!("refs/remotes/origin/{branch}"))
        .with_context(|| format!("Failed to resolve origin/{branch}"))
}