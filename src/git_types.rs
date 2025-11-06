use std::collections::BTreeMap;
use gix_url::Url;
use bstr::BStr;

pub type GitUrlBuf = Url;
pub type GitUrl = BStr;

pub type GitTags = Vec<String>;

pub type GitRemotes = BTreeMap<String, GitUrlBuf>;
