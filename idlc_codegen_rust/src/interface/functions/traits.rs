// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
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
