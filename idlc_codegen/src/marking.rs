// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkingStyle {
    Rust,
    C,
    Java,
}

impl MarkingStyle {
    const fn start(self) -> &'static str {
        match self {
            Self::Rust | Self::C => "",
            Self::Java => "/*\n",
        }
    }

    const fn end(self) -> &'static str {
        match self {
            Self::Rust | Self::C => "",
            Self::Java => "*/",
        }
    }

    const fn prefix(self) -> &'static str {
        match self {
            Self::Rust | Self::C => "//",
            Self::Java => "*",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marking(String);
impl std::fmt::Display for Marking {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::ops::Deref for Marking {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}
impl AsRef<str> for Marking {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Marking {
    pub fn add_marking(marking: Option<std::path::PathBuf>, style: MarkingStyle) -> Self {
        if let Some(marking) = marking {
            let mut documentation = style.start().to_string();
            let file = File::open(marking).expect("File not found");
            let reader = BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok(content) => {
                        documentation += style.prefix();
                        documentation.push(' ');
                        documentation.push_str(&content);
                    }
                    Err(e) => eprintln!("Reading file failed:\n{e}\n"),
                }
                documentation.push('\n');
            }
            documentation.push_str(style.end());
            documentation.push('\n');
            Self(documentation)
        } else {
            Self("".to_string())
        }
    }
}
