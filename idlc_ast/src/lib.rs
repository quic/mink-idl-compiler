pub mod ast;
pub mod pst;
#[cfg(test)]
mod tests;
pub mod visitor;

pub use ast::*;
pub use pst::Error;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Rc<Node>, Error> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| Error::Io(e, path.as_ref().display().to_string()))?;
    from_string(path.as_ref().to_path_buf(), content)
}

pub fn from_string<S: AsRef<str>>(root: PathBuf, s: S) -> Result<Rc<Node>, Error> {
    let pst = pst::parse(s.as_ref())?;
    Ok(Rc::new(Node::CompilationUnit(root, pst)))
}

pub fn dump<P: AsRef<Path>>(path: P) {
    use std::time::Instant;
    let now = Instant::now();
    let ast = from_file(path).unwrap();
    let duration = now.elapsed();
    println!("{ast:#?}");
    eprintln!("'dump_ast' completed in {duration:?}");
}
