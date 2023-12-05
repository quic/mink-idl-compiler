use idlc_codegen_c::interface::functions::invoke::Invoke;
use idlc_codegen_c::interface::variable_names::invoke::OP;

pub fn emit(
    function: &idlc_mir::Function,
    documentation: &str,
    signature: &idlc_codegen_c::interface::functions::signature::Signature,
    counts: &idlc_codegen::counts::Counter,
) -> String {
    let ident = &function.ident;
    let invoke = Invoke::new(function);
    let mut pre = invoke.pre();
    let mut post = invoke.post();
    let mut args = invoke.args();

    args = args.replace(" \\", "");
    pre = pre.replace(" \\", "");
    post = post.replace(" \\", "");

    let mut return_idents =
        idlc_codegen_c::interface::functions::signature::iter_to_string(signature.return_idents());
    if !return_idents.is_empty() {
        return_idents.remove(0);
    }

    let counts = (
        counts.input_buffers,
        counts.output_buffers,
        counts.input_objects,
        counts.output_objects,
    );

    format!(
        r#"
            {documentation}
            case {OP}_{ident}: {{
                if (k != ObjectCounts_pack{counts:?}{args}) {{
                    break;
                }}
                {pre}
                int32_t r = {ident}({return_idents});
                {post}
                return r;
            }} "#
    )
}
