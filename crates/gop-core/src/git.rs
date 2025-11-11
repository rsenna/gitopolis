use crate::gitopolis::GopError;

use git2::{Remote, Repository};
use std::path::Path;
use crate::gop_types::{GopUrl, GopUrlBuf};
use crate::remotes::GopRemotes;
// TODO: instead of "recreating" types like Remote[s] and such, why not just reuse types from gi2|gix?
// They are supposed to have a much better, memory efficient implementation. Besides, it's a waste of time and
// resources to keep converting from our internal representation against their(s).
// OTOH we'd pay the price of higher coupling - but that's arguably acceptable for a CLI tool.

// NOTE: Using AsRef extensively

pub trait Git {
	fn read_remote_url(&self, path: &Path, remote_name: &str) -> Result<GopRemote, GopError>;
	fn read_all_remotes(&self, path: &Path) -> Result<GopRemotes, GopError>;
	fn add_remote(&self, path: &Path, remote_name: &str, url: &GopUrl);
	fn clone(&self, path: &Path, url: &GopUrl) -> Result<(), GopError>;
}

pub struct GitImpl {}

impl Git for GitImpl {
	fn read_remote_url(&self, path: &Path, remote_name: &str) -> Result<GopRemote, GopError> {
		let repository = Repository::open(path)
			.map_err(|e| GopError::git_error(e))?;

		let remote: Remote = repository
			.find_remote(remote_name)
			.map_err(|e| GopError::remote_error(e, remote_name))?;

		let url_str = remote.url()
			.ok_or_else(|| GopError::remote_not_found(remote_name))?;

		let url = GopUrlBuf::try_from(url_str)
			.map_err(|e| GopError::remote_error(e, remote_name))?;

		let gop_remote = GopRemote::new(remote_name, url.path.as_ref())?;
		Ok(gop_remote)
	}

	fn read_all_remotes(&self, path: &Path) -> Result<GopRemotes, GopError> {
		let repository = Repository::open(path)
			.map_err(|e| GopError::Git {
				message: format!("Couldn't open git repo. {}", e.message()) })?;

		let remote_names = repository.remotes()
			.map_err(|e| GopError::Git {
				message: format!("Failed to read remotes. {}", e.message()) })?;

		let mut gop_remotes = GopRemotes::new();

		let result: Vec<Remote> = remote_names.iter()
			.filter_map(|remote_name| {
				remote_name.and_then(|rn| repository.find_remote(rn).ok())
			})
			.map(|r| GopRemote::from(r))
			.collect();

		for remote_name in remote_names.iter().flatten() {
			if let Ok(remote) = repository.find_remote(remote_name) {
				if let Some(url) = remote.url() {
					let gop_url_buf = GopUrlBuf::try_from(url.to_string())
						.map_err(|e| GopError::remote_error(e, remote_name))?;

					gop_remotes.insert(remote_name.to_string(), gop_url_buf);
				}
			}
		}

		Ok(gop_remotes)
	}

	fn add_remote<P: AsRef<Path>, S: AsRef<str>, U: AsRef<GopUrl>>(&self, path: P, remote_name: S, url: U) ->
		Result<(), GopError> {

		let git2_repo = git2::Repository::discover(path)
			.map_err(|e| GopError::git_error(e))?;

		git2_repo.remote_add_fetch(remote_name.as_ref(), url.as_ref().to_string().as_str())
			.map_err(|e| GopError::remote_error(e, remote_name.as_ref()))?;

		git2_repo.remote_add_push(remote_name.as_ref(), url.as_ref().to_string().as_str())
			.map_err(|e| GopError::remote_error(e, remote_name.as_ref()))?;

		Ok(())
	}

	fn clone<P: AsRef<Path>, U: AsRef<GopUrl>>(&self, path: P, url: U) -> Result<(), GopError> {
		let url_string = url.as_ref().to_string();

		if path.as_ref().exists() {
			println!("🏢 {}> Already exists, skipped.", path.as_ref().display());
			return Ok(());
		}

		println!("🏢 {}> Cloning {} ...", path.as_ref().display(), url_string.as_str());

		let _clone = Repository::clone(url_string.as_str(), path.as_ref())
			.map_err(|e| GopError::git_error(e))?;

		Ok(())
	}
}
