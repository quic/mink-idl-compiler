// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

pub fn emit(
    function: &idlc_mir::Function,
    documentation: &str,
    signature: &super::signature::Signature,
) -> String {
    let ident = &function.ident;
    let returns = signature.return_types().collect::<Vec<_>>().join(", ");
    let params = signature.params();
    format!(
        r#"
    {documentation}
    fn r#{ident}(&mut self, {params}) -> Result<({returns}), Error>
    "#
    )
}
