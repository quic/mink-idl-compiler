pub fn emit(
    function: &idlc_mir::Function,
    documentation: &str,
    signature: &super::signature::Signature,
) -> String {
    let ident = &function.ident;
    let returns = super::signature::iter_to_string(signature.return_types());
    let params = super::signature::iter_to_string(signature.params());
    format!(
        r#"
    {documentation}
    fn r#{ident}(&mut self, {params}) -> Result<({returns}), Error>
    "#
    )
}
