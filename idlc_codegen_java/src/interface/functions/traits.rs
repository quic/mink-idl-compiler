use std::borrow::Cow;

pub fn emit(
    function: &idlc_mir::Function,
    documentation: &str,
    signature: &super::signature::Signature,
) -> String {
    let ident = &function.ident;
    let params = super::signature::iter_to_string(signature.params());
    let (pre, post) = if function.is_optional() {
        (
            "default ",
            Cow::Owned(
                r"{ throw new IMinkObject.InvokeException(IMinkObject.ERROR_INVALID); }"
                    .to_string(),
            ),
        )
    } else {
        ("", Cow::Borrowed(";"))
    };

    format!(
        r#"{documentation}
    {pre}void {ident}({params}) throws IMinkObject.InvokeException{post}
    "#
    )
}
