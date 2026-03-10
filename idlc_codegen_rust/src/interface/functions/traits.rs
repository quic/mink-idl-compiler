// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use crate::interface::mink_primitives::GENERIC_ERROR;
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
    fn r#{ident}(&mut self, {params}) -> Result<({returns}), Error> {{
        Err({GENERIC_ERROR}::INVALID.into())
    }}
    "#
    )
}
