use std::borrow::Cow;
use bstr::BStr;
use derive_more::Display;
use gix_url::Url;

#[derive(Clone, Debug)]
pub struct GopTags<'a>(&'a [&'a str]);

#[derive(Clone, Debug)]
pub struct GopUrlBuf<'a>(&'a Url);

#[derive(Clone, Debug, Display)]
pub struct GopUrl<'a>(&'a BStr);

impl<'a> TryFrom<&'a Cow<'a, Url>> for GopUrl<'a> {
	type Error = ();

	fn try_from(value: &'a Url) -> Result<Self, Self::Error> {

	}
}
