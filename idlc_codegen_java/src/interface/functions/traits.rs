// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

pub fn emit(
    documentation: &str,
    signature: &super::signature::Signature,
    fn_ident: &str,
) -> String {
    let params = super::signature::iter_to_string(signature.params());
    format!(
        r#"{documentation}
    void {fn_ident}({params}) throws IMinkObject.InvokeException;
    "#
    )
}
