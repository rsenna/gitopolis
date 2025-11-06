use crate::gitopolis::GitopolisError;
use crate::gitopolis::GitopolisError::{GitError, GitRemoteError};
use crate::git_types::{GitRemotes, GitUrl, GitUrlBuf};

use git2::{Remote, Repository};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

// TODO: instead of "recreating" types like Remote[s] and such, why not just reuse types from gi2|gix?
// They are supposed to have a much better, memory efficient implementation. Besides, it's a waste of time and
// resources to keep converting from our internal representation against their(s).
// OTOH we'd pay the price of higher coupling - but that's arguably acceptable for a CLI tool.

// NOTE: Using AsRef extensively

pub trait Git {
	fn read_url<P: AsRef<Path>, R: AsRef<str>>(&self, path: P, remote_name: R) -> Result<GitUrlBuf, GitopolisError>;
	fn read_all_remotes<P: AsRef<Path>>(&self, path: P) -> Result<GitRemotes, GitopolisError>;
	fn add_remote<P: AsRef<Path>, S: AsRef<str>, U: AsRef<GitUrl>>(&self, path: P, remote_name: S, url: U);
	fn clone<P: AsRef<Path>, U: AsRef<GitUrl>>(&self, path: P, url: U) -> Result<(), GitopolisError>;
}

pub struct GitImpl {}

impl Git for GitImpl {
	fn read_url<P: AsRef<Path>, R: AsRef<str>>(&self, path: P, remote_name: R) -> Result<GitUrlBuf, GitopolisError> {
		let repository = Repository::open(path)
			.map_err(|e| GitopolisError::git_error(e))?;

		let remote: Remote = repository
			.find_remote(remote_name.as_ref())
			.map_err(|e| GitopolisError::remote_error(e, remote_name))?;

		let url_str = remote.url()
			.ok_or_else(|| GitopolisError::remote_not_found("remote is empty", ""))?;

		let url = GitUrlBuf::try_from(url_str)
			.map_err(|e| GitopolisError::remote_error(e, ""))?;

		Ok(url)
	}

	fn read_all_remotes<P: AsRef<Path>>(&self, path: P) -> Result<GitRemotes, GitopolisError> {
		let repository = Repository::open(path)
			.map_err(|e| GitError { message: format!("Couldn't open git repo. {}", e.message()) })?;

		let remote_names = repository.remotes()
			.map_err(|error| GitError { message: format!("Failed to read remotes. {}", error.message()) })?;

		let mut remotes = BTreeMap::new();

		for remote_name in remote_names.iter().flatten() {
			if let Ok(remote) = repository.find_remote(remote_name) {
				if let Some(url) = remote.url() {
					remotes.insert(remote_name.to_string(), url.to_string());
				}
			}
		}

		Ok(remotes)
	}

	fn add_remote(&self, path: &Path, remote_name: &str, url: &str) {
		let output = Command::new("git")
			.current_dir(path)
			.args(
				[
					"remote".to_string(),
					"add".to_string(),
					remote_name.to_string(),
					url.to_string(),
				]
				.to_vec(),
			)
			.output()
			.expect("Error running git remote add");
		if !output.status.success() {
			let stderr =
				String::from_utf8(output.stderr).expect("Error converting stderr to string");
			eprintln!("Warning: Failed to add remote {remote_name}: {stderr}");
		}
	}

	fn clone(&self, path: &Path, url: &str) -> Result<(), GitopolisError> {
		if Path::new(path).exists() {
			println!("🏢 {}> Already exists, skipped.", path.display());
			return Ok(());
		}
		println!("🏢 {}> Cloning {} ...", path.display(), url);
		let output = Command::new("git")
			.args(["clone".to_string(), url.to_string(), path.display().to_string()].to_vec())
			.output()
			.expect("Error running git clone");
		let stdout = String::from_utf8(output.stdout).expect("Error converting stdout to string");
		let stderr = String::from_utf8(output.stderr).expect("Error converting stderr to string");
		println!("{stdout}");
		println!("{stderr}");

		if !output.status.success() {
			return Err(GitError {
				message: format!("Failed to clone {} to {}", url, path.display()),
			});
		}

		Ok(())
	}
}
