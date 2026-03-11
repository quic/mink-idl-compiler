use idlc_ast::{SemanticVersion, VersionParseError};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedVersion {
    pub name: String,
    pub version: SemanticVersion,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("expected NAME@X.Y, NAME:X.Y, or NAME=X.Y (e.g. api@1.2)")]
    MissingSeparator,
    #[error("missing name before the separator (e.g. api@1.2)")]
    MissingName,
    #[error("missing version after the separator (e.g. api@1.2)")]
    MissingVersion,
    #[error("{0}")]
    Version(#[from] VersionParseError),
}

impl FromStr for NamedVersion {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Find the first separator occurrence among '@', ':', '='
        let (idx, _sep) = s
            .char_indices()
            .find(|&(_, c)| c == '@' || c == ':' || c == '=')
            .ok_or(ParseError::MissingSeparator)?;

        let (name, ver) = s.split_at(idx);
        let ver = &ver[1..]; // drop the separator char

        let name = name.trim();
        let ver = ver.trim();

        if name.is_empty() {
            return Err(ParseError::MissingName);
        }
        if ver.is_empty() {
            return Err(ParseError::MissingVersion);
        }

        let version = ver.parse::<SemanticVersion>()?;

        Ok(NamedVersion {
            name: name.to_owned(),
            version,
        })
    }
}
