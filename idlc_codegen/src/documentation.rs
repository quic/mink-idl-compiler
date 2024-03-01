#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentationStyle {
    Rust,
    C,
    Java,
}

impl DocumentationStyle {
    const fn start(self) -> &'static str {
        match self {
            Self::Rust => "///<pre>",
            Self::C => "/*",
            Self::Java => "/**",
        }
    }

    const fn end(self) -> &'static str {
        match self {
            Self::Rust => "///</pre>",
            Self::C | Self::Java => "*/",
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
    #[must_use]
    pub fn new(function: &idlc_mir::Function, style: DocumentationStyle) -> Self {
        function.doc.as_ref().map_or_else(
            || Self(String::new()),
            |doc| Self::new_with_idlc_doc(doc, style),
        )
    }

    fn new_with_idlc_doc(doc: &str, style: DocumentationStyle) -> Self {
        let mut documentation = style.start().to_string();
        documentation.push('\n');
        let indent = doc.find('*').unwrap_or(0);

        for line in doc.lines() {
            let docstring = line.get(indent..).unwrap_or_default();

            documentation += style.prefix();
            if !docstring.is_empty() {
                documentation.push(' ');
                documentation.push_str(docstring.trim_end());
            }
            documentation.push('\n');
        }
        documentation.push_str(style.end());

        Self(documentation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DOCUMENTATION: &str = r"
* Hello this is a sample documentation
    I can even contain no asterisk in the beginning and this is style a valid idl doc style   

* @param[out]
* New Lines must be preserved and convention interleaving should work too!
* @param[in] credentials  Lines that wrap around must ensure
                          formatting is maintained
                          a
                          b
                          c";

    #[test]
    fn rust() {
        let documentation =
            Documentation::new_with_idlc_doc(DOCUMENTATION, DocumentationStyle::Rust);
        assert_eq!(
            documentation.as_ref(),
            r"///<pre>
///
///  Hello this is a sample documentation
///    I can even contain no asterisk in the beginning and this is style a valid idl doc style
///
///  @param[out]
///  New Lines must be preserved and convention interleaving should work too!
///  @param[in] credentials  Lines that wrap around must ensure
///                          formatting is maintained
///                          a
///                          b
///                          c
///</pre>"
        );
    }

    #[test]
    fn c() {
        let documentation = Documentation::new_with_idlc_doc(DOCUMENTATION, DocumentationStyle::C);
        assert_eq!(
            documentation.as_ref(),
            r"/*
*
*  Hello this is a sample documentation
*    I can even contain no asterisk in the beginning and this is style a valid idl doc style
*
*  @param[out]
*  New Lines must be preserved and convention interleaving should work too!
*  @param[in] credentials  Lines that wrap around must ensure
*                          formatting is maintained
*                          a
*                          b
*                          c
*/"
        );
    }

    #[test]
    fn java() {
        let documentation =
            Documentation::new_with_idlc_doc(DOCUMENTATION, DocumentationStyle::Java);
        assert_eq!(
            documentation.as_ref(),
            r"/**
*
*  Hello this is a sample documentation
*    I can even contain no asterisk in the beginning and this is style a valid idl doc style
*
*  @param[out]
*  New Lines must be preserved and convention interleaving should work too!
*  @param[in] credentials  Lines that wrap around must ensure
*                          formatting is maintained
*                          a
*                          b
*                          c
*/"
        );
    }
}
