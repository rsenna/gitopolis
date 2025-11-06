use crate::gitopolis::GitopolisError;
use crate::git_types::{GitUrl, GitUrlBuf, GitTags, GitRemotes};

use log::info;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use derive_builder::Builder;

const INVALID_REPO_NAME: &'static str = "Repository name is invalid, please specify a distinct one";

type Remotes = BTreeMap<String, Remote>;
type VRepo = Vec<Repo>;

trait RemotesArgEx {
	fn into_remotes(self) -> Remotes;
}

impl RemotesArgEx for GitRemotes {
	fn into_remotes(self) -> Remotes {
		let mut remotes = BTreeMap::new();

		for (name, url) in self {
			let remote = Remote { name: name.clone(), url };
			remotes.insert(name, remote);
		}

		remotes
	}
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Repos {
	repos: VRepo,
}

#[derive(Debug, Deserialize, Serialize, Clone, Builder)]
#[builder(setter(into))]
pub struct Repo {
	pub path: PathBuf,
	#[builder(default = "self.default_name()?")]
	pub name: String,
	pub tags: GitTags,
	pub remotes: Remotes,
}

impl RepoBuilder {
	fn default_name(&self) -> Result<String, String> {
		match self.path {
			Some(ref path) => Ok(Repo::get_name_from_path(path.as_path())),
			_ => Err(String::from(INVALID_REPO_NAME)),
		}
	}
}

impl Repo {
	pub fn new(path: &Path) -> Result<Self, String> {
		RepoBuilder::default()
			.path(path)
			.build().map_err(|e| format!("Failed to create repo: {}", e))
	}

	pub fn new_with_path_and_remotes(path: &Path, remotes_arg: GitRemotes) -> Result<Self, String> {
		RepoBuilder::default()
			.path(path)
			.remotes(remotes_arg.into_remotes())
			.build().map_err(|e|  format!("Failed to create repo: {}", e))
	}

	pub(crate) fn get_name_from_path(path: &Path) -> String {
		 path.file_name()
			.map(|s| s.to_string_lossy().to_string())
			.expect(INVALID_REPO_NAME)
	}

	pub(crate) fn add_remote(&mut self, remote: Remote) {
		self.remotes.insert(remote.name.clone(), remote);
	}
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Remote {
	pub name: String,
	pub url: GitUrlBuf,
}

impl Repos {
	pub fn as_vec(&self) -> &VRepo {
		&self.repos
	}

	pub fn into_vec(self) -> VRepo {
		self.repos
	}

	pub fn new() -> Self {
		Default::default()
	}

	pub fn new_with_repos(repos: VRepo) -> Self {
		Repos { repos }
	}

	pub fn find<F>(&mut self, predicate: F) -> Option<&mut Repo>
	where
		F: FnMut(&&mut Repo) -> bool,
	{
		self.repos.iter_mut().find(predicate)
	}

	pub fn find_by_name(&mut self, name: &str) -> Option<&mut Repo> {
		self.find(|r| r.name == name)
	}

	pub fn find_by_path(&mut self, path: &Path) -> Option<&mut Repo> {
		self.find(|r| r.path == path)
	}
	pub fn index_by_name(&self, name: &str) -> Option<usize> {
		self.repos.iter().position(|r| r.name == name)
	}

	pub fn index_by_path(&self, path: &Path) -> Option<usize> {
		self.repos.iter().position(|r| r.path.eq(path))
	}

	pub fn add(&mut self, repo: Repo) {
		let repo_path = repo.path.clone();
		self.repos.push(repo);
		self.repos.sort_by(|a, b| a.path.cmp(&b.path));
		info!("Added {}", repo_path.display());
	}

	pub fn add_new_repo(&mut self, path: &Path, remotes_arg: GitRemotes) -> Result<(), String> {
		let repo = Repo::new_with_path_and_remotes(path, remotes_arg)?;
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
	) -> Result<(), GitopolisError> {
		fn action(tag_name: &str, repo: &mut Repo) {
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
	) -> Result<(), GitopolisError> {
		fn action(tag_name: &str, repo: &mut Repo) {
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
	) -> Result<(), GitopolisError>
	where
		P: Fn(&str, &mut Repo) {
		for repo_name in repo_names {
			let repo = self.find_by_name(repo_name.as_str()).ok_or_else(|| {
				GitopolisError::StateError {
					message: format!("Repo '{repo_name}' not found"),
				}
			})?;

			action(tag_name, repo);
		}

		Ok(())
	}
}

#[test]
fn idempotent_tag() -> Result<(), String> {
	let mut repos = Repos::new();

	let path = Path::new("some/path/to/repo");
	let mut remotes_arg: GitRemotes = BTreeMap::new();

	let url = GitUrlBuf::try_from("url").map_err(|e| format!("Failed to read Git URL: {}", e))?;

	remotes_arg.insert("origin".into(), url);
	let remotes = remotes_arg.into_remotes();

	let repo = RepoBuilder::default()
		.path(path)
		.remotes(remotes)
		.build()
		.map_err(|e| e.to_string())?;

	let tag = "tag_name";

	repos
		.add_tag(tag, vec![repo.name.clone()])
		.expect("add_tag failed");

	repos
		.add_tag(tag, vec![repo.name.clone()])
		.expect("add_tag failed");

	let repo = repos.find_by_path(path).expect("repo awol");

	assert_eq!(1, repo.tags.len());
	assert_eq!(tag, repo.tags[0]);

	Ok(())
}
