#[derive(Debug, Clone)]
pub struct EscapedIdent<'a>(&'a idlc_mir::Ident);
impl<'a> std::fmt::Display for EscapedIdent<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r#{}", &self.0)
    }
}

impl<'a> EscapedIdent<'a> {
    #[inline]
    pub fn new(ident: &'a idlc_mir::Ident) -> Self {
        EscapedIdent(ident)
    }
}

impl<'a> From<&'a idlc_mir::Ident> for EscapedIdent<'a> {
    #[inline]
    fn from(value: &'a idlc_mir::Ident) -> Self {
        Self::new(value)
    }
}
