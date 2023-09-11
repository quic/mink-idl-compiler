pub mod ast;
pub mod pst;
#[cfg(test)]
mod tests;
pub mod visitor;

pub use ast::*;
pub use pst::Error;
use std::path::{Path, PathBuf};

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Ast, Error> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| Error::Io(e, path.as_ref().display().to_string()))?;
    from_string(path.as_ref().to_path_buf(), content)
}

pub fn from_string<S: AsRef<str>>(root: PathBuf, s: S) -> Result<Ast, Error> {
    let nodes = pst::parse_to_ast(s.as_ref())?;
    Ok(Ast { tag: root, nodes })
}

pub fn dump<P: AsRef<Path>>(path: P) {
    use std::time::Instant;
    let now = Instant::now();
    let ast = from_file(path).unwrap();
    let duration = now.elapsed();
    println!("{ast:#?}");
    eprintln!("'dump_ast' completed in {duration:?}");
}
