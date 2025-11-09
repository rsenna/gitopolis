#[allow(unused_imports)]
// BUG: SomeError import *is* required, but IDE says otherwise.
use crate::gitopolis::{GopError, SomeError};
use crate::gop_types::{GopUrl, GopUrlBuf, GopTags, GopRemoteUrls};

use log::info;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::vec::IntoIter;
use bstr::ByteSlice;
use derive_builder::Builder;

type GopRemotes = BTreeMap<String, GopRemote>;
type GopRepoVec = Vec<GopRepo>;

trait GopRemoteUrlsEx {
	fn into_remotes(self) -> GopRemotes;
}

impl GopRemoteUrlsEx for GopRemoteUrls {
	fn into_remotes(self) -> GopRemotes {
		let mut remotes = GopRemotes::new();

		for (name, url) in self {
			let remote = GopRemote { name: name.clone(), url };
			remotes.insert(name, remote);
		}

		remotes
	}
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GopRepos {
	repos: GopRepoVec,
}

#[derive(Debug, Deserialize, Serialize, Clone, Builder)]
#[builder(setter(into))]
pub struct GopRepo {
	pub path: PathBuf,
	#[builder(default = "self.default_name()?")]
	pub name: String,
	pub tags: GopTags,
	pub remotes: GopRemotes,
}

// TODO: Use GopError instead of String

impl GopRepoBuilder {
	fn default_name(&self) -> Result<String, GopRepoBuilderError> {
		self.path.as_ref()
			.and_then(|p| GopRepo::get_name_from_path(p.as_path()))
			.ok_or_else(|| GopRepoBuilderError::from("Could not extract name from path".to_string()))
	}
}

impl GopRepo {
	pub fn new(path: &Path) -> Result<GopRepo, GopError> {
		GopRepoBuilder::default()
			.path(path)
			.build().map_err(|e| GopError::state_error(e))
	}

	pub fn new_with_path_and_remotes(path: &Path, remotes_arg: GopRemoteUrls) -> Result<Self, GopError> {
		GopRepoBuilder::default()
			.path(path)
			.remotes(remotes_arg.into_remotes())
			.build().map_err(|e| GopError::remote_error(e, "(multiple)") )
	}

	/// Extracts the repository name from a git URL or local path, to determine the path name
	/// that git clone would use. Handles SSH, HTTPS URLs, and local paths.
	///
	/// Examples:
	/// - git@github.com:user/repo.git -> repo
	/// - https://github.com/user/repo.git -> repo
	/// - https://github.com/user/repo -> repo
	/// - https://dev.azure.com/org/project/_git/myrepo -> myrepo
	/// - some/repository/path -> path
	/// - some/repository/path.git -> path
	/// - source_repo -> source_repo
	/// - C:\path\to\repo.git -> repo (Windows)
	pub(crate) fn get_name_from_path_or_url<S: AsRef<str>>(path_or_url: S) -> Option<String> {
		let path = PathBuf::from(path_or_url.as_ref());
		Self::get_name_from_path(path)
	}

	pub(crate) fn get_name_from_path<P: AsRef<Path>>(path: P) -> Option<String> {
		path.as_ref().file_name()
			.map(|s| s.to_string_lossy().to_string())
			.filter(|s| !s.is_empty() && s.ends_with(".git"))
	}

	pub(crate) fn get_name_from_url<U: AsRef<GopUrl>>(url: U) -> Option<String> {
		let path = PathBuf::from(url.as_ref().to_string());
		Self::get_name_from_path(path)
	}

	pub(crate) fn add_remote(&mut self, remote: GopRemote) {
		self.remotes.insert(remote.name.clone(), remote);
	}

	pub(crate) fn replace_remotes(&mut self, remotes_arg: GopRemoteUrls) {
		self.remotes.clear();
		let remotes = remotes_arg.into_remotes();
		self.remotes.extend(remotes);
	}
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GopRemote {
	pub name: String,
	pub url: GopUrlBuf,
}

impl GopRemote {
	pub fn new(name: &str, url: &GopUrl) -> Result<Self, GopError> {
		let name = name.to_string();
		let url = GopUrlBuf::try_from(url)
			.map_err(|e| GopError::state_error(e))?;

		Ok(Self { name, url })
	}
}

impl IntoIterator for GopRepos {
	type Item = GopRepo;
	type IntoIter = IntoIter<GopRepo>;

	fn into_iter(self) -> Self::IntoIter {
		self.repos.into_iter()
	}
}

impl <'repo> TryFrom<git2::Remote<'repo>> for GopRemote {
	type Error = GopError;

	fn try_from(value: git2::Remote<'repo>) -> Result<Self, Self::Error> {
		let name = value.name().ok_or_else(|| GopError::remote_not_found(""))?.to_string();
		let url_str = value.url().ok_or_else(|| GopError::remote_not_found(&name))?;
		let url = GopUrlBuf::try_from(url_str)
			.map_err(|e| GopError::remote_error(e, &name))?;

		let remote = GopRemote::new(name.as_str(), url.to_bstring().as_bstr())?;

		Ok(remote)
	}
}

impl GopRepos {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn new_with_repos(repos: GopRepoVec) -> Self {
		GopRepos { repos }
	}

	pub fn find<F>(&mut self, predicate: F) -> Option<&mut GopRepo>
	where
		F: FnMut(&&mut GopRepo) -> bool,
	{
		self.repos.iter_mut().find(predicate)
	}

	pub fn find_by_name(&mut self, name: &str) -> Option<&mut GopRepo> {
		self.find(|r| r.name == name)
	}

	pub fn find_by_path(&mut self, path: &Path) -> Option<&mut GopRepo> {
		self.find(|r| r.path == path)
	}

	pub fn index_by_name(&self, name: &str) -> Option<usize> {
		self.repos.iter().position(|r| r.name == name)
	}

	pub fn index_by_path(&self, path: &Path) -> Option<usize> {
		self.repos.iter().position(|r| r.path.eq(path))
	}

	pub fn add(&mut self, repo: GopRepo) {
		let repo_path = repo.path.clone();
		self.repos.push(repo);
		self.repos.sort_by(|a, b| a.path.cmp(&b.path));
		info!("Added {}", repo_path.display());
	}

	pub fn add_new_repo(&mut self, path: &Path, remotes_arg: GopRemoteUrls) -> Result<(), GopError> {
		let repo = GopRepo::new_with_path_and_remotes(path, remotes_arg)?;
		Ok(self.add(repo))
	}

	pub fn remove_by_names(&mut self, repo_names: Vec<String>) {
		for repo_name in repo_names {
			match self.index_by_name(repo_name.as_str()) {
				Some(ix) => {
					self.repos.remove(ix);
				}
				None => {
					info!("Repo already absent, skipped: {repo_name}")
				}
			}
		}
	}

	pub fn add_tag(
		&mut self,
		tag_name: &str,
		repo_names: Vec<String>
	) -> Result<(), GopError> {
		fn action(tag_name: &str, repo: &mut GopRepo) {
			if !repo.tags.iter().any(|s| s.as_str() == tag_name) {
				repo.tags.push(tag_name.to_string());
				repo.tags.sort_by_key(|a| a.to_lowercase());
			}
		}

		self.do_tag(tag_name, repo_names, action)
	}

	pub fn remove_tag(
		&mut self,
		tag_name: &str,
		repo_names: Vec<String>,
	) -> Result<(), GopError> {
		fn action(tag_name: &str, repo: &mut GopRepo) {
			if let Some(ix) = repo.tags.iter().position(|t| t == tag_name) {
				repo.tags.remove(ix);
			}
		}

		self.do_tag(tag_name, repo_names, action)
	}

	fn do_tag<P>(
		&mut self,
		tag_name: &str,
		repo_names: Vec<String>,
		action: P,
	) -> Result<(), GopError>
	where
		P: Fn(&str, &mut GopRepo) {
		for repo_name in repo_names {
			let repo = self.find_by_name(repo_name.as_str()).ok_or_else(|| {
				GopError::State {
					message: format!("Repo '{repo_name}' not found"),
				}
			})?;

			action(tag_name, repo);
		}

		Ok(())
	}
}

#[test]
fn idempotent_tag() -> Result<(), GopError> {
	let mut repos = GopRepos::new();

	let path = Path::new("some/path/to/repo");
	let mut remotes_arg: GopRemoteUrls = BTreeMap::new();

	const NAME: &str = "some-url";
	const URL: &str = "some-url.git";
	const ORIGIN: &str = "some-origin";
	const TAG: &str = "some_tag";

	let url = GopUrlBuf::try_from(URL)
		.map_err(|e| GopError::state_error_invalid_url(e, URL))?;

	remotes_arg.insert("origin".into(), url);
	let remotes = remotes_arg.into_remotes();

	let repo = GopRepoBuilder::default()
		.path(path)
		.remotes(remotes)
		.build()
		.map_err(|e| GopError::state(e.message()))?;

	assert_eq!(repo.name, NAME);

	repos.add_tag(TAG, vec![NAME.into()])
		.expect("add_tag failed");

	repos.add_tag(TAG, vec![NAME.into()])
		.expect("add_tag failed");

	let repo = repos.find_by_path(path)
		.expect("repo awol");

	assert_eq!(1, repo.tags.len());
	assert_eq!(TAG, repo.tags[0]);

	Ok(())
}
