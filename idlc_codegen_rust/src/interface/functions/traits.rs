use std::borrow::Cow;

use crate::interface::mink_primitives::GENERIC_ERROR;
pub fn emit(
    function: &idlc_mir::Function,
    documentation: &str,
    signature: &super::signature::Signature,
) -> String {
    let ident = &function.ident;
    let returns = super::signature::iter_to_string(signature.return_types());
    let params = super::signature::iter_to_string(signature.params());
    let definition = if function.is_optional() {
        Cow::Owned(format!("{{ Err({GENERIC_ERROR}::INVALID.into()) }}"))
    } else {
        Cow::Borrowed(";")
    };
    format!(
        r#"
    {documentation}
    fn r#{ident}(&mut self, {params}) -> Result<({returns}), Error>{definition}"#
    )
}
