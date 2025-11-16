use std::ops::Deref;
use std::path::PathBuf;
use bstr::{BStr, BString, ByteSlice};
use derive_more::Display;
use gix_url::{parse::Error as GixError, Url as GixUrl};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct VaqTagsBuf {
	items: Vec<String>
}

#[derive(Clone, Debug, Display)]
pub struct VaqUrl<'a>(pub &'a BStr);

impl<'a> VaqUrl<'a> {
	pub fn new<B: AsRef<BStr>>(text: &'a B) -> VaqUrl<'a> {
		VaqUrl::<'a>(text.as_ref())
	}
}

impl Deref for VaqUrl<'_> {
	type Target = BStr;

	fn deref(&self) -> &Self::Target {
		self.0.as_bstr()
	}
}

#[derive(Clone, Debug, Display)]
#[display("{text}")]
pub struct VaqUrlBuf {
	pub url: GixUrl,
	text: BString,
}

#[derive(Debug, Error)]
pub enum VaqUrlBufError {
	#[error("Failed to parse URL from BString: {0}")]
	ParseBString(BString, GixError),

	#[error("Failed to parse URL from String: {0}")]
	ParseString(String, GixError),
}

impl VaqUrlBuf {
	pub fn new(url: &VaqUrl) -> VaqUrlBuf {
		let gix_url: GixUrl = GixUrl::try_from(url.0).expect("infallible");
		let text = url.0.to_owned();
		VaqUrlBuf { url: gix_url, text }
	}
}

impl From<GixUrl> for VaqUrlBuf {
	fn from(url: GixUrl) -> Self {
		let text = url.to_bstring();
		VaqUrlBuf { url, text }
	}
}

impl TryFrom<VaqUrl<'_>> for VaqUrlBuf {
	type Error = VaqUrlBufError;

	fn try_from(value: VaqUrl) -> Result<Self, Self::Error> {
		let url = value.as_bstr().to_owned();
		Self::try_from(url)
	}
}

impl TryFrom<BString> for VaqUrlBuf {
	type Error = VaqUrlBufError;

	fn try_from(value: BString) -> Result<Self, Self::Error> {
		let url: GixUrl = GixUrl::try_from(value.as_bstr())
			.map_err(|e| VaqUrlBufError::ParseBString(value.clone(), e))?;

		Ok(VaqUrlBuf { url, text: value })
	}
}

impl TryFrom<&str> for VaqUrlBuf {
	type Error = VaqUrlBufError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let url: GixUrl = GixUrl::try_from(value)
			.map_err(|e| VaqUrlBufError::ParseString(value.to_owned(), e))?;

		Ok(VaqUrlBuf::from(url))
	}
}

impl From<VaqUrlBuf> for PathBuf {
	fn from(vaq_url: VaqUrlBuf) -> Self {
		let path = vaq_url.url.path.to_string();
		PathBuf::from(path)
	}
}
