pub fn emit(
    function: &idlc_mir::Function,
    documentation: &str,
    signature: &super::signature::Signature,
) -> String {
    let ident = &function.ident;
    let params = super::signature::iter_to_string(signature.params());
    format!(
        r#"{documentation}
    default void {ident}({params}) throws IMinkObject.InvokeException {{ throw new IMinkObject.InvokeException(IMinkObject.ERROR_INVALID); }}
    "#
    )
}
