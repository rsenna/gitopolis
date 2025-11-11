use crate::gop_types::GopUrl;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::vec::IntoIter;
use derive_more::Display;
use git2::Remote;
use thiserror::Error;

#[derive(Clone, Debug, Display)]
#[display("({name}, {url})")]
pub struct GopRemote<'a> {
	#[display("{}", self.display_name())]
	pub name: Cow<'a, str>,

	#[display("{}", self.display_url())]
	pub url: Cow<'a, GopUrl<'a>>,
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum GopRemoteError {
	#[error("Remote name is missing")]
	MissingName,
	#[error("Remote URL is missing")]
	MissingUrl,
	#[error("Invalid remote URL: {0}")]
	InvalidUrl(String, gix_url::parse::Error),
	#[error("Git error: {0}")]
	GitError(String),
}

impl <'a> GopRemote<'a> {
	pub fn new(name: &'a str, url: &'a GopUrl) -> Self {
		let url = Cow::<'a, GopUrl>::Borrowed(url);

		GopRemote {
			name: name.into(),
			url: url.into()
		}
	}

	pub fn new_cow(name: Cow<'a, str>, url: Cow<'a, GopUrl<'a>>) -> Self {
		Self {
			name,
			url
		}
	}

	pub fn display_name(&self) -> &str {
		&self.name
	}

	pub fn display_url(&self) -> &GopUrl {
		&self.url
	}
}

impl <'a> TryFrom<&'a Remote<'a>> for GopRemote<'a> {
	type Error = GopRemoteError;

	fn try_from(value: &'a Remote) -> Result<Self, Self::Error> {
		let name: &'a str = value.name().ok_or_else(|| GopRemoteError::MissingName)?;
		let url: &'a str = value.url().ok_or_else(|| GopRemoteError::MissingUrl)?;
		let url = GopUrl::<'a>::try_from(url)
			.map_err(|e| GopRemoteError::InvalidUrl(url.to_string(), e))?;



		Ok(GopRemote::new_cow(name.into(), url.into()))
	}
}

#[derive(Clone, Debug)]
pub struct GopRemotes<'a> {
	pub items: BTreeMap<&'a str, &'a GopRemote<'a>>,
}

impl <'a> GopRemotes<'a> {
	pub fn new() -> Self {
		GopRemotes { items: BTreeMap::new() }
	}
}

impl <'a> IntoIterator for GopRemotes<'a> {
	type Item = &'a GopRemote<'a>;
	type IntoIter = IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		let vec: Vec<Self::Item> = self.items.into_iter()
			.map(|(_, v)| v)
			.collect();

		vec.into_iter()
	}
}
