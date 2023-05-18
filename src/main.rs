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
}

fn main() {
    use clap::Parser as ClapParser;
    use pest::Parser as PestParser;
    let args = Args::parse();
    let file = std::fs::read_to_string(args.path).expect("File read to succeed.");

    let now = std::time::Instant::now();
    let ast = IDLParser::parse(Rule::idl, &file).expect("Successful AST dump");
    let end = now.elapsed();
    println!("AST generated in {end:?}");
    std::hint::black_box(ast);
    //dbg!(ast);
}
