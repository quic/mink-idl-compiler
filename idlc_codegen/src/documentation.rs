#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentationStyle {
    Rust,
    C,
    Java,
}

impl DocumentationStyle {
    fn apply_replacements(self, doc: &str) -> std::borrow::Cow<'_, str> {
        // Rust uses Markdown so we need to replace `[` or intra-doc links break
        if self == Self::Rust && (doc.contains(']') || doc.contains(']')) {
            std::borrow::Cow::Owned(doc.replace(']', "\\]").replace('[', "\\["))
        } else {
            std::borrow::Cow::Borrowed(doc)
        }
    }

    const fn start(self) -> Option<&'static str> {
        match self {
            Self::Rust => None,
            Self::C => Some("/*"),
            Self::Java => Some("/**"),
        }
    }

    const fn end(self) -> Option<&'static str> {
        match self {
            Self::Rust => None,
            Self::C | Self::Java => Some("*/"),
        }
    }

    const fn prefix(self) -> &'static str {
        match self {
            Self::Rust => "///",
            Self::C | Self::Java => "*",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Documentation(String);
impl std::fmt::Display for Documentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::ops::Deref for Documentation {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}
impl AsRef<str> for Documentation {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Documentation {
    pub fn new(function: &idlc_mir::Function, style: DocumentationStyle) -> Self {
        function.doc.as_ref().map_or_else(
            || Self(String::new()),
            |doc| Self::new_with_idlc_doc(doc, style),
        )
    }

    fn new_with_idlc_doc(doc: &str, style: DocumentationStyle) -> Self {
        let mut documentation = String::new();
        if let Some(start) = style.start() {
            documentation.push_str(start);
            documentation.push('\n');
        }

        for line in doc.lines() {
            let line = line.trim_start();
            let docstring = line.strip_prefix('*').unwrap_or(line);

            documentation += style.prefix();
            if !docstring.is_empty() {
                documentation.push(' ');
                documentation.push_str(&style.apply_replacements(docstring.trim()));
            }
            documentation.push('\n');
        }
        if let Some(end) = style.end() {
            documentation.push_str(end);
            documentation.push('\n');
        }

        Self(documentation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DOCUMENTATION: &str = r#"* Hello this is a sample documentation
      I can even contain no asterisk in the beginning and this is style a valid idl doc style
    *
    * @param[out]
    * New Lines must be preserved and convention interleaving should work too!"#;

    #[test]
    fn rust() {
        let documentation =
            Documentation::new_with_idlc_doc(DOCUMENTATION, DocumentationStyle::Rust);
        assert_eq!(
            documentation.as_ref(),
            r#"/// Hello this is a sample documentation
/// I can even contain no asterisk in the beginning and this is style a valid idl doc style
///
/// @param\[out\]
/// New Lines must be preserved and convention interleaving should work too!
"#
        );
    }

    #[test]
    fn c() {
        let documentation = Documentation::new_with_idlc_doc(DOCUMENTATION, DocumentationStyle::C);
        assert_eq!(
            documentation.as_ref(),
            r#"/*
* Hello this is a sample documentation
* I can even contain no asterisk in the beginning and this is style a valid idl doc style
*
* @param[out]
* New Lines must be preserved and convention interleaving should work too!
*/
"#
        );
    }

    #[test]
    fn java() {
        let documentation =
            Documentation::new_with_idlc_doc(DOCUMENTATION, DocumentationStyle::Java);
        assert_eq!(
            documentation.as_ref(),
            r#"/**
* Hello this is a sample documentation
* I can even contain no asterisk in the beginning and this is style a valid idl doc style
*
* @param[out]
* New Lines must be preserved and convention interleaving should work too!
*/
"#
        );
    }
}
