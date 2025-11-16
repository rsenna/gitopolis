use crate::vaq_types::{VaqUrl, VaqUrlBuf};

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::error::Error;
use std::rc::Rc;
use std::vec::IntoIter;

use bstr::{BStr, BString};
use derive_more::{Display, IntoIterator};
use git2::Remote;
use gix_url::parse::Error as GixUrlError;
use thiserror::Error;

#[derive(Clone, Debug, Display)]
#[display("({name}) {url}")]
pub struct VaqRemote {
	#[display("{}")]
	pub name: Rc<String>,

	#[display("{}")]
	pub url: VaqUrlBuf,
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum VaqRemoteError {
	#[error("Remote name is missing")]
	MissingName,

	#[error("Remote URL is missing")]
	MissingUrl,

	#[error("Invalid remote URL: {0}")]
	InvalidUrl(String, Box<dyn Error>),

	#[error("Git error: {0}")]
	GitError(String),
}

impl VaqRemote {
	pub fn new(name: String, url: VaqUrlBuf) -> Self {
		VaqRemote { name: Rc::from(name), url }
	}
}

impl <'a> TryFrom<&'a Remote<'a>> for VaqRemote {
	type Error = VaqRemoteError;

	fn try_from(value: &'a Remote) -> Result<Self, Self::Error> {
		let name = value.name().ok_or_else(|| VaqRemoteError::MissingName)?.to_string();
		let url_str = value.url().ok_or_else(|| VaqRemoteError::MissingUrl)?;
		let url_bstr = BString::new(url_str.into());
		let url = VaqUrlBuf::try_from(url_bstr)
			.map_err(|e| VaqRemoteError::InvalidUrl(url_str.into(), e.into()))?;

		Ok(VaqRemote::new(name, url))
	}
}

#[derive(Clone, Debug, IntoIterator)]
pub struct VaqRemoteSlice<'a>(pub &'a [VaqRemote]);

#[derive(Clone, Debug)]
pub struct VaqRemotes {
	pub(crate) items: BTreeMap<Rc<String>, VaqRemote>,
}

impl VaqRemotes {
	pub fn new() -> Self {
		VaqRemotes { items: BTreeMap::new() }
	}
}

impl<'a> From<VaqRemoteSlice<'a>> for VaqRemotes {
	fn from(value: VaqRemoteSlice<'a>) -> Self {
		let mut remotes = VaqRemotes::new();

		for remote_ref in value {
			let remote = remote_ref.clone();
			let name = remote.name.clone();

			remotes.items.insert(name, remote);
		}

		remotes
	}
}

impl IntoIterator for VaqRemotes {
	type Item = VaqRemote;
	type IntoIter = IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		let vec: Vec<Self::Item> = self.items.into_iter()
			.map(|(_, v)| v)
			.collect();

		vec.into_iter()
	}
}
