// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

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
            Self::Java => "*/\n",
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
    pub fn new(marking: &str, style: MarkingStyle) -> Self {
        if marking.is_empty() {
            return Self(String::new());
        }
        let mut documentation = style.start().to_string();
        for line in marking.lines() {
            documentation += style.prefix();
            documentation.push(' ');
            documentation.push_str(line);
            documentation.push('\n');
        }
        documentation.push_str(style.end());
        documentation.push('\n');
        Self(documentation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MARKING: &str = "Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
All rights reserved.
Confidential and Proprietary - Qualcomm Technologies, Inc.
";

    #[test]
    fn rust() {
        let marking = Marking::new(MARKING, MarkingStyle::Rust);
        assert_eq!(
            marking.as_ref(),
            r"// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// All rights reserved.
// Confidential and Proprietary - Qualcomm Technologies, Inc.

"
        );
    }

    #[test]
    fn c() {
        let marking = Marking::new(MARKING, MarkingStyle::C);
        assert_eq!(
            marking.as_ref(),
            r"// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// All rights reserved.
// Confidential and Proprietary - Qualcomm Technologies, Inc.

"
        );
    }

    #[test]
    fn java() {
        let marking = Marking::new(MARKING, MarkingStyle::Java);
        assert_eq!(
            marking.as_ref(),
            r"/*
* Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
* All rights reserved.
* Confidential and Proprietary - Qualcomm Technologies, Inc.
*/

"
        );
    }
}
