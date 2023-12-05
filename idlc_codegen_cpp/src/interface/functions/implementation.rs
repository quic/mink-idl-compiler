use idlc_codegen_c::interface::functions::implementation::Implementation;
use idlc_codegen_c::interface::variable_names::invoke::{ARGS, OP};

pub fn emit(
    function: &idlc_mir::Function,
    documentation: &str,
    counts: &idlc_codegen::counts::Counter,
    signature: &idlc_codegen_c::interface::functions::signature::Signature,
) -> String {
    let ident = &function.ident;
    let total = counts.total();

    let mut params =
        idlc_codegen_c::interface::functions::signature::iter_to_string(signature.params());
    if !params.is_empty() {
        params.remove(0);
    }

    let implementation = Implementation::new(function, counts);
    let mut initializations = implementation.initializations();
    let mut post_call_assignments = implementation.post_call_assignments();
    initializations = initializations.replace('\n', "\n    ");
    post_call_assignments = post_call_assignments.replace('\n', "\n    ");

    let returns = if total > 0 {
        format!(
            "invoke({OP}_{ident}, a, ObjectCounts_pack({0}, {1}, {2}, {3}));",
            counts.input_buffers,
            counts.output_buffers,
            counts.input_objects,
            counts.output_objects
        )
    } else {
        format!("invoke({OP}_{ident}, 0, 0);")
    };

    format!(
        r#"
    {documentation}
    virtual int32_t {ident}({params}) {{
        ObjectArg {ARGS}[{total}]={{{{{{0,0}}}}}};
        {initializations}
        int32_t result = {returns}
        {post_call_assignments}

        return result;
    }}
    "#
    )
}
