mod ast;
mod passes;

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
    /// Path of IDL to dump AST for
    path: String,

    #[arg(long, value_enum)]
    /// Dump various phases of the compiler and exit.
    dump: Option<Dumpable>,
}

fn check<T: std::fmt::Debug, E: std::fmt::Display>(r: Result<T, E>) -> T {
    match r {
        Ok(t) => {
            dbg!(&t);
            t
        }
        Err(e) => {
            eprintln!("{e}");
            panic!();
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
enum Dumpable {
    /// Parse Syntax Tree
    Pst,
    /// Abstract Syntax Tree
    Ast,
}

fn main() {
    use clap::Parser as ClapParser;
    let args = Args::parse();
    if let Some(dump) = args.dump {
        match dump {
            Dumpable::Pst => ast::dump_pst(&args.path),
            Dumpable::Ast => ast::dump(&args.path),
        }
        std::process::exit(0);
    }

    let ast = ast::Node::from_file(&args.path).unwrap();

    println!("Checking for duplicate symbols...");
    check(passes::duplicate::contains_duplicate_symbols(&ast));

    println!("Checking for unresolved includes...");
    let includes = passes::includes::Includes::new(&ast);
    check(includes.symbol_table());
}
