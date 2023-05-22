#[cfg(test)]
mod tests;

#[derive(pest_derive::Parser)]
#[grammar = "../grammar/idl.pest"]
pub struct IDLParser;

#[derive(clap::Parser)]
#[command(author, version, about = None, long_about)]
/// Parse .idl files into AST.
///
/// TODO(arvimuku): Currently the syntatic analysis phase is done although
/// there's no semantic analysis done for external types. I'm wondering how i'd
/// address the dependencies nicely especially detect cyclic dependencies and
/// such would need to include some sort of cycle detection + topological
/// sorting + creating a giant paste table.
///
/// The C-compiler for example returns `nested too deeply`, rust detects cyclic
/// imports nicer.
struct Args {
    #[arg(short, long)]
    /// Path of IDL to dump AST for
    path: String,

    #[arg(short, long, default_value = "false")]
    /// Print the Abstract Syntax Tree formed from the IDL. Useful to analyze parsing
    /// inconsistencies..
    dump_ast: bool,
}

fn main() {
    use clap::Parser as ClapParser;
    use pest::Parser as PestParser;
    let args = Args::parse();
    let file = std::fs::read_to_string(args.path).expect("File read to succeed.");
    let ast = match IDLParser::parse(Rule::idl, &file) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{e}");
            panic!("Couldn't generate AST");
        }
    };
}
