//! Remote repository management - cloning, syncing, and cleanup

use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks, Repository as GitRepo};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use url::Url;

use crate::config::Config;
use crate::error::{AppError, Result};

/// Progress callback for clone/fetch operations
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send>;

/// Parse a GitHub URL or shorthand into a normalized HTTPS URL
pub fn parse_github_url(input: &str) -> Result<(String, String, String)> {
    // Handle shorthand format: owner/repo
    if !input.contains("://") && !input.starts_with("git@") {
        if let Some((owner, repo)) = input.split_once('/') {
            let repo = repo.trim_end_matches(".git");
            let url = format!("https://github.com/{owner}/{repo}.git");
            return Ok((url, owner.to_string(), repo.to_string()));
        }
        return Err(AppError::Other(format!(
            "Invalid repository format: {input}. Use owner/repo or full URL."
        )));
    }

    // Handle SSH format: git@github.com:owner/repo.git
    if input.starts_with("git@") {
        let parts: Vec<&str> = input.split(':').collect();
        if parts.len() == 2 {
            let path = parts[1].trim_end_matches(".git");
            if let Some((owner, repo)) = path.split_once('/') {
                let url = format!("https://github.com/{owner}/{repo}.git");
                return Ok((url, owner.to_string(), repo.to_string()));
            }
        }
        return Err(AppError::Other(format!("Invalid SSH URL format: {input}")));
    }

    // Handle HTTPS URL
    let parsed =
        Url::parse(input).map_err(|e| AppError::Other(format!("Invalid URL: {input} - {e}")))?;

    let path = parsed
        .path()
        .trim_start_matches('/')
        .trim_end_matches(".git");
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() >= 2 {
        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        let url = format!("https://github.com/{owner}/{repo}.git");
        Ok((url, owner, repo))
    } else {
        Err(AppError::Other(format!(
            "Cannot extract owner/repo from URL: {input}"
        )))
    }
}

/// Get the path where remote repos are cloned
pub fn get_repos_dir() -> Result<PathBuf> {
    let config_dir = Config::config_dir()?;
    Ok(config_dir.join("repos"))
}

/// Get the clone path for a specific remote repo
pub fn get_clone_path(owner: &str, repo: &str) -> Result<PathBuf> {
    let repos_dir = get_repos_dir()?;
    Ok(repos_dir.join(owner).join(repo))
}

/// Clone a remote repository
pub fn clone_repository(
    url: &str,
    target_path: &Path,
    branch: Option<&str>,
    shallow: bool,
    progress_cb: Option<ProgressCallback>,
) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Set up progress callbacks
    let received = Arc::new(AtomicUsize::new(0));
    let total = Arc::new(AtomicUsize::new(0));
    let cancel = Arc::new(AtomicBool::new(false));

    let received_clone = received.clone();
    let total_clone = total.clone();

    let mut callbacks = RemoteCallbacks::new();

    callbacks.transfer_progress(move |progress| {
        received_clone.store(progress.received_objects(), Ordering::Relaxed);
        total_clone.store(progress.total_objects(), Ordering::Relaxed);
        true
    });

    // Set up credentials callback for token auth
    callbacks.credentials(|_url, username_from_url, allowed_types| {
        // Try SSH key first
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            if let Some(username) = username_from_url {
                return git2::Cred::ssh_key_from_agent(username);
            }
        }

        // Try token from environment
        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            if let Ok(token) =
                std::env::var("KDEX_GITHUB_TOKEN").or_else(|_| std::env::var("GITHUB_TOKEN"))
            {
                return git2::Cred::userpass_plaintext("x-access-token", &token);
            }
        }

        // Default credentials
        git2::Cred::default()
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    if shallow {
        fetch_opts.depth(1);
    }

    // Progress reporting thread
    let received_report = received.clone();
    let total_report = total.clone();
    let cancel_report = cancel.clone();

    if progress_cb.is_some() {
        std::thread::spawn(move || {
            while !cancel_report.load(Ordering::Relaxed) {
                let r = received_report.load(Ordering::Relaxed);
                let t = total_report.load(Ordering::Relaxed);
                if let Some(ref cb) = progress_cb {
                    cb(r, t, "Cloning...");
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    }

    // Clone the repository
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_opts);

    if let Some(b) = branch {
        builder.branch(b);
    }

    let result = builder.clone(url, target_path);
    cancel.store(true, Ordering::Relaxed);

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            // Clean up failed clone
            let _ = std::fs::remove_dir_all(target_path);
            Err(AppError::Other(format!("Clone failed: {e}")))
        }
    }
}

/// Sync (fetch + reset) a remote repository
pub fn sync_repository(repo_path: &Path, branch: Option<&str>) -> Result<bool> {
    let repo = GitRepo::open(repo_path)
        .map_err(|e| AppError::Other(format!("Failed to open repository: {e}")))?;

    // Get the remote
    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| AppError::Other(format!("Failed to find origin remote: {e}")))?;

    // Set up credentials callback
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, allowed_types| {
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            if let Some(username) = username_from_url {
                return git2::Cred::ssh_key_from_agent(username);
            }
        }
        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            if let Ok(token) =
                std::env::var("KDEX_GITHUB_TOKEN").or_else(|_| std::env::var("GITHUB_TOKEN"))
            {
                return git2::Cred::userpass_plaintext("x-access-token", &token);
            }
        }
        git2::Cred::default()
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    // Fetch from origin
    let refspecs: &[&str] = &[];
    remote
        .fetch(refspecs, Some(&mut fetch_opts), None)
        .map_err(|e| AppError::Other(format!("Fetch failed: {e}")))?;

    // Get the target branch
    let branch_name = branch.unwrap_or("HEAD");
    let fetch_head = repo
        .find_reference(&format!("refs/remotes/origin/{branch_name}"))
        .or_else(|_| repo.find_reference("FETCH_HEAD"))
        .map_err(|e| AppError::Other(format!("Failed to find fetch head: {e}")))?;

    let target_commit = fetch_head
        .peel_to_commit()
        .map_err(|e| AppError::Other(format!("Failed to get commit: {e}")))?;

    // Check if there are changes
    let head = repo.head().ok();
    let current_oid = head.as_ref().and_then(git2::Reference::target);
    let new_oid = target_commit.id();

    if current_oid == Some(new_oid) {
        return Ok(false); // No changes
    }

    // Reset to the fetched commit (hard reset)
    repo.reset(target_commit.as_object(), git2::ResetType::Hard, None)
        .map_err(|e| AppError::Other(format!("Reset failed: {e}")))?;

    Ok(true) // Changes were made
}

/// Delete a cloned repository directory
pub fn delete_clone(repo_path: &Path) -> Result<()> {
    if repo_path.exists() {
        std::fs::remove_dir_all(repo_path)
            .map_err(|e| AppError::Other(format!("Failed to delete clone: {e}")))?;
    }

    // Also try to remove empty parent directories
    if let Some(parent) = repo_path.parent() {
        let _ = std::fs::remove_dir(parent); // Ignore error if not empty
    }

    Ok(())
}

/// Check if a path is inside the repos directory (i.e., a remote clone)
pub fn is_remote_clone(path: &Path) -> Result<bool> {
    let repos_dir = get_repos_dir()?;
    Ok(path.starts_with(&repos_dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_shorthand() {
        let (url, owner, repo) = parse_github_url("rust-lang/rust").unwrap();
        assert_eq!(url, "https://github.com/rust-lang/rust.git");
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust");
    }

    #[test]
    fn test_parse_github_https() {
        let (url, owner, repo) = parse_github_url("https://github.com/rust-lang/rust.git").unwrap();
        assert_eq!(url, "https://github.com/rust-lang/rust.git");
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust");
    }

    #[test]
    fn test_parse_github_https_no_git() {
        let (url, owner, repo) = parse_github_url("https://github.com/rust-lang/rust").unwrap();
        assert_eq!(url, "https://github.com/rust-lang/rust.git");
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust");
    }

    #[test]
    fn test_parse_github_ssh() {
        let (url, owner, repo) = parse_github_url("git@github.com:rust-lang/rust.git").unwrap();
        assert_eq!(url, "https://github.com/rust-lang/rust.git");
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust");
    }

    #[test]
    fn test_invalid_format() {
        assert!(parse_github_url("invalid").is_err());
    }
}
