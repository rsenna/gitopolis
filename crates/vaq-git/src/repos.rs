use std::collections::BTreeMap;
use crate::vaq_types::{VaqUrl, VaqUrlBuf, VaqTagsBuf};
use crate::remotes::{VaqRemote, VaqRemoteSlice, VaqRemotes};

use std::path::{Path, PathBuf};
use std::vec::IntoIter;

use bstr::ByteSlice;
use derive_builder::Builder;
use log::info;
use thiserror::Error;

type VaqRepoVec = Vec<VaqRepo>;

#[derive(Debug, Default)]
pub struct VaqRepos {
	items: VaqRepoVec,
}

#[derive(Builder, Clone, Debug)]
#[builder(setter(into))]
pub struct VaqRepo {
	pub path: PathBuf,

	#[builder(default = "self.default_name()?")]
	pub name: String,

	pub tags: VaqTagsBuf,
	pub remotes: VaqRemotes,
}

impl VaqRepoBuilder {
	fn default_name(&self) -> Result<String, VaqRepoBuilderError> {
		let name = self.path.as_ref()
			.and_then(|p| p.file_name())
			.and_then(|os| os.to_str())
			.ok_or_else(|| VaqRepoBuilderError::UninitializedField("name"))?
			.to_string();

		Ok(name)
	}
}

#[derive(Debug, Error)]
#[non_exhaustive]
enum VaqRepoError {
	#[error("Failed to create repository")]
	CannotCreateRepo { #[source] source: VaqRepoBuilderError },

	#[error("Failed to determine repository name from path")]
	InvalidRepoNameFromUrl { url: VaqUrlBuf, error: VaqRepoBuilderError },
}

impl VaqRepo {
	pub fn new(path: &Path) -> Result<VaqRepo, VaqRepoBuilderError> {
		VaqRepoBuilder::default()
			.path(path)
			.build()
	}

	/// Extracts the repository name from a git URL or local path to determine the path name
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
	pub(crate) fn get_name_from_url(url: &VaqUrl) -> Result<String, VaqRepoError> {
		let url_buf: VaqUrlBuf = VaqUrlBuf::new(url);
		let path = url_buf.url.path.to_string();
		let path_buf = PathBuf::from(path);

		path_buf.file_name()
			.and_then(|os| os.to_str())
			.map(|name| name.to_string())
			.ok_or_else(|| VaqRepoError::InvalidRepoNameFromUrl {
				url: url_buf,
				error: VaqRepoBuilderError::UninitializedField("name"),
			})
	}

	pub(crate) fn add_remote(&mut self, remote: VaqRemote) {
		self.remotes.items.insert(
			remote.name.clone(),
			remote
		);
	}

	pub(crate) fn replace_remotes(&mut self, remotes_arg: VaqRemoteSlice) {
		self.remotes.items.clear();
		let remotes: VaqRemotes = remotes_arg.into();
		self.remotes.items.extend(remotes.items);
	}
}

impl IntoIterator for VaqRepos {
	type Item = VaqRepo;
	type IntoIter = IntoIter<VaqRepo>;

	fn into_iter(self) -> Self::IntoIter {
		self.items.into_iter()
	}
}

impl VaqRepos {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn new_with_repos(repos: VaqRepoVec) -> Self {
		VaqRepos { items: repos }
	}

	pub fn find<F>(&mut self, predicate: F) -> Option<&mut VaqRepo>
	where
		F: FnMut(&&mut VaqRepo) -> bool,
	{
		self.items.iter_mut().find(predicate)
	}

	pub fn find_by_name(&mut self, name: &str) -> Option<&mut VaqRepo> {
		self.find(|r| r.name == name)
	}

	pub fn find_by_path(&mut self, path: &Path) -> Option<&mut VaqRepo> {
		self.find(|r| r.path == path)
	}

	pub fn index_by_name(&self, name: &str) -> Option<usize> {
		self.items.iter().position(|r| r.name == name)
	}

	pub fn index_by_path(&self, path: &Path) -> Option<usize> {
		self.items.iter().position(|r| r.path.eq(path))
	}

	pub fn add(&mut self, repo: VaqRepo) {
		let repo_path = repo.path.clone();
		self.items.push(repo);
		self.items.sort_by(|a, b| a.path.cmp(&b.path));
		info!("Added {}", repo_path.display());
	}

	pub fn add_new_repo(&mut self, path: &Path, remotes_arg: VaqRemoteSlice) -> Result<(), VaqError> {
		let repo = VaqRepo::new_with_path_and_remotes(path, remotes_arg)?;
		Ok(self.add(repo))
	}

	pub fn remove_by_names(&mut self, repo_names: Vec<String>) {
		for repo_name in repo_names {
			match self.index_by_name(repo_name.as_str()) {
				Some(ix) => {
					self.items.remove(ix);
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
	) -> Result<(), VaqError> {
		fn action(tag_name: &str, repo: &mut VaqRepo) {
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
	) -> Result<(), VaqError> {
		fn action(tag_name: &str, repo: &mut VaqRepo) {
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
	) -> Result<(), VaqError>
	where
		P: Fn(&str, &mut VaqRepo) {
		for repo_name in repo_names {
			let repo = self.find_by_name(repo_name.as_str()).ok_or_else(|| {
				VaqError::State {
					message: format!("Repo '{repo_name}' not found"),
				}
			})?;

			action(tag_name, repo);
		}

		Ok(())
	}
}

#[test]
fn idempotent_tag() -> Result<(), VaqError> {
	let mut repos = VaqRepos::new();

	let path = Path::new("some/path/to/repo");
	let mut remotes_arg: VaqRemoteSlice = BTreeMap::new();

	const NAME: &str = "some-url";
	const URL: &str = "some-url.git";
	const ORIGIN: &str = "some-origin";
	const TAG: &str = "some_tag";

	let url = VaqUrlBuf::try_from(URL)
		.map_err(|e| VaqError::state_error_invalid_url(e, URL))?;

	remotes_arg.insert("origin".into(), url);
	let remotes = remotes_arg.into_remotes();

	let repo = VaqRepoBuilder::default()
		.path(path)
		.remotes(remotes)
		.build()
		.map_err(|e| VaqError::state(e.message()))?;

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
