use crate::git::Git;
use crate::repos::{GopRepo, GopRepos};
use crate::storage::Storage;
use crate::tag_filter::TagFilter;

use log::info;
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::io;
use std::path::{Path, PathBuf};
use crate::gop_types::{GopTags, GopUrl};

pub struct Gitopolis {
	storage: Box<dyn Storage>,
	git: Box<dyn Git>,
}

#[derive(Debug)]
pub enum GopError {
	Git { message: String },
	State { message: String },
	Remote { message: String, remote: String },
	Io { inner: io::Error },
}

pub trait SomeError {
	fn message(&self) -> String;
}

impl SomeError for GopError {
	fn message(&self) -> String {
		match self {
			GopError::Git { message } => message.to_string(),
			GopError::State { message } => message.to_string(),
			GopError::Remote { message, remote: _ } => message.to_string(),
			GopError::Io { inner } => inner.to_string(),
		}
	}
}

impl SomeError for git2::Error {
	fn message(&self) -> String { format!("{}", self) }
}

impl SomeError for gix_url::parse::Error {
	fn message(&self) -> String { format!("{}", self) }
}

impl SomeError for Infallible {
	fn message(&self) -> String { format!("{}", self) }
}

// TODO: find some way of avoid code duplication for GopError helper methods
impl GopError {
	pub fn git<S: AsRef<str>>(message: S) -> Self {
		GopError::Git {
			message: format!("{}", message.as_ref())
		}
	}

	pub fn git_cannot_open<S: AsRef<str>>(message: S) -> Self {
		Self::git(format!("Couldn't open git repo: {}", message.as_ref()))
	}

	pub fn git_error<E: SomeError>(error: E) -> Self {
		Self::git(error.message())
	}

	pub fn remote<S: AsRef<str>, R: AsRef<str>>(message: S, remote_name: R) -> Self {
		GopError::Remote {
			message: format!("Error with remote '{}': {}", remote_name.as_ref(), message.as_ref()),
			remote: remote_name.as_ref().to_string(),
		}
	}

	pub fn remote_not_found<R: AsRef<str>>(remote_name: R) -> Self {
		GopError::remote("Not found.", remote_name)
	}

	pub fn remote_error<E: SomeError, S: AsRef<str>>(error: E, remote_name: S) -> Self {
		Self::remote(error.message(), remote_name)
	}

	pub fn state<S: AsRef<str>>(message: S) -> Self {
		Self::State {
			message: format!("{}", message.as_ref().to_string())
		}
	}

	pub fn state_error<E: SomeError>(error: E) -> Self {
		Self::state(error.message())
	}

	pub fn state_error_invalid_url<E: SomeError, S: AsRef<GopUrl>>(error: E, url: S) -> Self {
		Self::state(format!("Invalid Git URL {}: {}.", url.as_ref(), error.message()))
	}

	pub fn state_invalid_url<S: AsRef<GopUrl>>(url: S) -> Self {
		Self::state(format!("Invalid Git URL {}.", url.as_ref()))
	}

	pub fn state_repo_not_found<S: AsRef<str>>(repo_id: S) -> Self {
		Self::state(format!("Repo '{}' not found.", repo_id.as_ref()))
	}

	pub fn state_error_repo_not_found<E: SomeError, S: AsRef<str>>(error: E, repo_id: S) -> Self {
		Self::state(format!("Repo '{}' not found: {}.", repo_id.as_ref(), error.message()))
	}
}

impl Gitopolis {
	pub fn new(storage: Box<dyn Storage>, git: Box<dyn Git>) -> Self {
		Self { storage, git }
	}

	pub fn add(&mut self, repo_path: &Path) -> Result<(), GopError> {
		let mut repos = self.load()?;
		let normalized_path = normalize_path(repo_path);

		if repos.index_by_path(normalized_path.as_path()).is_some() {
			info!("{} already added, ignoring.", normalized_path.display());
			return Ok(());
		}

		let remotes = self.git.read_all_remotes(normalized_path.as_path())?;

		repos.add_new_repo(normalized_path.as_path(), remotes)
			.map_err(GopError::state_error)?;

		self.save(repos)?;
		Ok(())
	}

	pub fn remove_repos_by_name(&mut self, repo_names: &[String]) -> Result<(), GopError> {
		let mut repos = self.load()?;
		repos.remove_by_names(repo_names.to_vec());
		self.save(repos)
	}

	pub fn add_tag(
		&mut self,
		tag_name: &str,
		repo_names: &[String],
	) -> Result<(), GopError> {
		let mut repos = self.load()?;
		repos.add_tag(tag_name, repo_names.to_vec())?;
		self.save(repos)
	}

	pub fn remove_tag(
		&mut self,
		tag_name: &str,
		repo_paths: &[String],
	) -> Result<(), GopError> {
		let mut repos = self.load()?;
		repos.remove_tag(tag_name, repo_paths.to_vec())?;
		self.save(repos)
	}

	/// Filter repos by tag filter with AND/OR logic.
	pub fn list(&self, filter: &TagFilter) -> Result<Vec<GopRepo>, GopError> {
		let repos = self.load()?;

		let mut result: Vec<GopRepo> = repos
			.into_iter()
			.filter(|repo| filter.matches(&repo.tags))
			.collect();

		result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
		Ok(result)
	}

	pub fn read(&self) -> Result<GopRepos, GopError> {
		self.load()
	}

	pub fn clone(&self, repos: Vec<GopRepo>) -> Result<usize, GopError> {
		let mut error_count = 0;

		for repo in repos {
			// Determine which remote to use for cloning (prefer origin)
			let clone_remote_name = if repo.remotes.contains_key("origin") {
				"origin"
			} else {
				repo.remotes.keys().next().map(|s| s.as_str()).unwrap_or("")
			};

			if let Some(clone_remote) = repo.remotes.get(clone_remote_name) {
				// Clone the repo
				match self.git.clone(repo.path.as_ref(), clone_remote.url.to_bstring()) {
					Ok(()) => {
						// Add all other remotes
						for (name, remote) in &repo.remotes {
							if name != clone_remote_name {
								self.git.add_remote(repo.path.as_ref(), name, remote.url.to_bstring())?;
							}
						}
					}

					Err(_) => {
						eprintln!("Warning: Could not clone {}", repo.path.display());
						error_count += 1;
					}
				}
			}
		}

		if error_count > 0 {
			eprintln!("{error_count} repos failed to clone");
			std::process::exit(1);
		}

		Ok(error_count)
	}

	pub fn tags(&self) -> Result<GopTags, GopError> {
		let repos = self.load()?;

		let mut tags: GopTags = repos.into_iter()
			.flat_map(|r| r.tags)
			.collect();

		tags.sort();
		tags.dedup();

		Ok(tags)
	}

	pub fn sync_read_remotes(&mut self, filter: &TagFilter) -> Result<(), GopError> {
		let mut repos = self.load()?;
		let repo_list = self.list(filter)?;
		let mut error_count = 0;

		for repo in repo_list {
			match self.git.read_all_remotes(repo.path.as_ref()) {
				Ok(remotes) => {
					// Find the repo in the mutable repos structure and update its remotes
					if let Some(repo_mut) = repos.find_by_path(repo.path.as_path()) {
						repo_mut.replace_remotes(remotes);

						info!("Updated {} with remotes from git", repo.path.display());
					}
				}

				Err(_) => {
					eprintln!("Warning: Could not read remotes from {}", repo.path.display());
					error_count += 1;
				}
			}
		}

		self.save(repos)?;

		if error_count > 0 {
			eprintln!("{error_count} repos failed to sync");
			std::process::exit(1);
		}

		Ok(())
	}

	pub fn sync_write_remotes(&self, filter: &TagFilter) -> Result<(), GopError> {
		let repo_list = self.list(filter)?;
		let mut error_count = 0;

		for repo in repo_list {

			// Get current remotes from git
			let current_remotes = match self.git.read_all_remotes(repo.path.clone()) {
				Ok(remotes) => remotes,

				Err(_) => {
					eprintln!("Warning: Could not write remotes to {}", repo.path.display());
					error_count += 1;
					continue;
				}
			};

			// Add any missing remotes from config
			for (name, remote) in &repo.remotes {
				if !current_remotes.contains_key(name) {
					self.git.add_remote(repo.path.as_ref(), name.as_ref(), remote.url.to_bstring())?;
					info!("Added remote {} to {}", name.as_ref(), repo.path.display());
				}
			}
		}

		if error_count > 0 {
			eprintln!("{error_count} repos failed to sync");
			std::process::exit(1);
		}

		Ok(())
	}

	fn show_by_name(&self, repo_name: &str) -> Result<Option<GopRepo>, GopError> {
		todo!()
	}

	fn show_by_path(&self, repo_path: &Path) -> Result<Option<GopRepo>, GopError> {
		let mut repos = self.load()?;
		let normalized_path = normalize_path(repo_path);

		let repo = repos.find_by_path(&normalized_path);
		let repo = repos
			.into_iter()
			.find(|r| r.path == normalized_path)
			.ok_or_else(|| GopError::state_error_repo_not_found(e, repo_path.as_ref()));

		Ok(RepoInfo {
			path: repo.path.clone(),
			tags: repo.tags.clone(),
			remotes: repo.remotes.clone(),
		})
	}

	pub fn clone_and_add<U: AsRef<GopUrl>, P: AsRef<Path>, T: AsRef<GopTags>>(
		&mut self,
		url: U,
		target_path: P,
		tags: T,
	) -> Result<String, GopError> {
		// Use target_path if provided, otherwise extract from URL
		let path_name = match target_path {
			Some(path) => path.to_string(),
			None => extract_repo_name_from_url(url).ok_or_else(|| StateError {
				message: format!("Could not extract repository name from URL: {}", url),
			})?,
		};

		// Clone the repository
		self.git.clone(&path_name, url)?;

		// Add the repository to gitopolis
		self.add(path_name.clone())?;

		// Add tags if any were specified
		if !tags.is_empty() {
			for tag in tags {
				self.add_tag(tag.as_str(), std::slice::from_ref(&path_name))?;
			}
		}

		Ok(path_name)
	}

	pub fn move_repo(&mut self, old_path: &str, new_path: &str) -> Result<(), GopError> {
		let mut repos = self.load()?;
		let normalized_old = normalize_path(old_path.to_string());
		let normalized_new = normalize_path(new_path.to_string());

		// Find the repo in the config
		let repo = repos
			.as_vec()
			.iter()
			.find(|r| r.path == normalized_old)
			.ok_or_else(|| StateError {
				message: format!("Repo '{}' not found", normalized_old),
			})?
			.clone();

		// Create parent paths if they don't exist
		if let Some(parent) = std::path::Path::new(&normalized_new).parent() {
			if !parent.as_os_str().is_empty() {
				std::fs::create_dir_all(parent).map_err(|e| IoError { inner: e })?;
			}
		}

		// Move the actual path on the filesystem
		std::fs::rename(&normalized_old, &normalized_new).map_err(|e| IoError { inner: e })?;

		// Update the config: remove old entry and add new one with same tags/remotes
		repos.remove_by_names(vec![normalized_old]);
		repos.add_with_tags_and_remotes(normalized_new, repo.tags, repo.remotes);

		self.save(repos)?;
		Ok(())
	}

	fn save(&self, repos: GopRepos) -> Result<(), GopError> {
		let state_toml = serialize(&repos)?;
		self.storage.save(state_toml);
		Ok(())
	}

	fn load(&self) -> Result<GopRepos, GopError> {
		if !self.storage.exists() {
			return Ok(GopRepos::new());
		}

		let state_toml = self.storage.read();

		parse(&state_toml)
	}
}

fn serialize(repos: &GopRepos) -> Result<String, GopError> {
	toml::to_string(&repos).map_err(|error| StateError {
		message: format!("Failed to generate toml for repo list. {error}"),
	})
}

fn parse(state_toml: &str) -> Result<GopRepos, GopError> {
	let mut named_container: BTreeMap<String, Vec<GopRepo>> =
		toml::from_str(state_toml).map_err(|error| StateError {
			message: format!("Failed to parse state data as valid TOML. {error}"),
		})?;

	let repos = named_container
		.remove("repos") // [re]move this rather than taking a ref so that ownership moves with it (borrow checker)
		.expect("Failed to read 'repos' entry from state TOML");
	Ok(GopRepos::new_with_repos(repos))
}

fn normalize_paths(repo_paths: &[PathBuf]) -> Vec<PathBuf> {
	repo_paths
		.iter()
		.map(|f| normalize_path(f))
		.collect()
}

fn normalize_path(repo_path: &Path) -> PathBuf {
	let mut result = repo_path.to_path_buf();

	if result.ends_with("/") || result.ends_with("\\") {
		result.pop();
	}

	result
}

#[test]
fn test_extract_repo_name_from_url() {
	assert_eq!(
		extract_repo_name_from_url("git@github.com:user/repo.git"),
		Some("repo".to_string())
	);
	assert_eq!(
		extract_repo_name_from_url("https://github.com/user/repo.git"),
		Some("repo".to_string())
	);
	assert_eq!(
		extract_repo_name_from_url("https://github.com/user/repo"),
		Some("repo".to_string())
	);
	assert_eq!(
		extract_repo_name_from_url("git@gitlab.com:group/subgroup/project.git"),
		Some("project".to_string())
	);
	assert_eq!(
		extract_repo_name_from_url("https://dev.azure.com/org/project/_git/myrepo"),
		Some("myrepo".to_string())
	);
	// Simple local path
	assert_eq!(
		extract_repo_name_from_url("source_repo"),
		Some("source_repo".to_string())
	);
	// Windows path
	assert_eq!(
		extract_repo_name_from_url("C:\\Users\\test\\repo.git"),
		Some("repo".to_string())
	);
	// Windows path without .git extension
	assert_eq!(
		extract_repo_name_from_url("C:\\Temp\\myrepo"),
		Some("myrepo".to_string())
	);
}

#[test]
fn test_normalize_paths() {
	let input = vec![
		"foo".to_string(),
		"bar/".to_string(),  // *nix
		"baz\\".to_string(), // windows
	];
	let output = normalize_paths(&input);
	assert_eq!(output, vec!["foo", "bar", "baz"]);
}
