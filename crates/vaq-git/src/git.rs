use crate::vaq_types::{VaqUrl, VaqUrlBuf, VaqUrlBufError};
use crate::remotes::{VaqRemote, VaqRemotes};

use git2::{Error as Git2Error, Remote, Repository};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub trait Git {
	fn read_remote_url(&self, path: &Path, remote_name: &str) -> Result<VaqRemote, VaqError>;
	fn read_all_remotes(&self, path: &Path) -> Result<VaqRemotes, VaqError>;
	fn add_remote(&self, path: &Path, remote_name: &str, url: &VaqUrl);
	fn clone(&self, path: &Path, url: &VaqUrl) -> Result<(), VaqError>;
}

pub struct GitImpl {}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum GitError {
	#[error("Invalid git repository path: {0}")]
	InvalidPath(PathBuf, Git2Error),

	#[error("Remote '{0}' not found")]
	InvalidRemoteName(String, Git2Error),

	#[error("Missing remote URL for '{0}'")]
	MissingRemoteUrl(String),

	#[error("Invalid remote URL for '{0}': {1}")]
	InvalidRemoteUrl(String, String, VaqUrlBufError),
}

impl Git for GitImpl {
	fn read_remote_url<'a>(&self, path: &Path, remote_name: &str) -> Result<VaqRemote, GitError> {
		let repository = Repository::open(path)
			.map_err(|e| GitError::InvalidPath(path.to_owned(), e))?;

		let remote: Remote = repository
			.find_remote(remote_name)
			.map_err(|e| GitError::InvalidRemoteName(remote_name.to_owned(), e))?;

		let url_str = remote.url()
			.ok_or_else(|| GitError::MissingRemoteUrl(remote_name.to_owned()))?;

		let url = VaqUrlBuf::try_from(url_str)
			.map_err(|e| GitError::InvalidRemoteUrl(remote_name.to_owned(), url_str.to_owned(), e))?;

		let vaq_remote = VaqRemote::new(remote_name, url.as_ref());
		Ok(vaq_remote)
	}

	fn read_all_remotes(&self, path: &Path) -> Result<VaqRemotes, VaqError> {
		let repository = Repository::open(path)
			.map_err(|e| VaqError::Git {
				message: format!("Couldn't open git repo. {}", e.message()) })?;

		let remote_names = repository.remotes()
			.map_err(|e| VaqError::Git {
				message: format!("Failed to read remotes. {}", e.message()) })?;

		let mut vaq_remotes = VaqRemotes::new();

		let result: Vec<Remote> = remote_names.iter()
			.filter_map(|remote_name| {
				remote_name.and_then(|rn| repository.find_remote(rn).ok())
			})
			.map(|r| VaqRemote::from(r))
			.collect();

		for remote_name in remote_names.iter().flatten() {
			if let Ok(remote) = repository.find_remote(remote_name) {
				if let Some(url) = remote.url() {
					let vaq_url_buf = VaqUrlBuf::try_from(url.to_string())
						.map_err(|e| VaqError::remote_error(e, remote_name))?;

					vaq_remotes.insert(remote_name.to_string(), vaq_url_buf);
				}
			}
		}

		Ok(vaq_remotes)
	}

	fn add_remote<P: AsRef<Path>, S: AsRef<str>, U: AsRef<VaqUrl>>(&self, path: P, remote_name: S, url: U) ->
		Result<(), VaqError> {

		let git2_repo = git2::Repository::discover(path)
			.map_err(|e| VaqError::git_error(e))?;

		git2_repo.remote_add_fetch(remote_name.as_ref(), url.as_ref().to_string().as_str())
			.map_err(|e| VaqError::remote_error(e, remote_name.as_ref()))?;

		git2_repo.remote_add_push(remote_name.as_ref(), url.as_ref().to_string().as_str())
			.map_err(|e| VaqError::remote_error(e, remote_name.as_ref()))?;

		Ok(())
	}

	fn clone<P: AsRef<Path>, U: AsRef<VaqUrl>>(&self, path: P, url: U) -> Result<(), VaqError> {
		let url_string = url.as_ref().to_string();

		if path.as_ref().exists() {
			println!("🏢 {}> Already exists, skipped.", path.as_ref().display());
			return Ok(());
		}

		println!("🏢 {}> Cloning {} ...", path.as_ref().display(), url_string.as_str());

		let _clone = Repository::clone(url_string.as_str(), path.as_ref())
			.map_err(|e| VaqError::git_error(e))?;

		Ok(())
	}
}
