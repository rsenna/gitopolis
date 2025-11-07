use std::collections::BTreeMap;
use gix_url::Url;
use bstr::BStr;

// TODO: turn these into newtypes

pub type GopRemoteUrl = (String, GopUrlBuf);
pub type GopRemoteUrls = BTreeMap<String, GopUrlBuf>;

pub type GopTags = Vec<String>;

pub type GopUrlBuf = Url;
pub type GopUrl = BStr;

