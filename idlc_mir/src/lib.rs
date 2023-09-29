pub mod mir;

pub use idlc_ast::pst::Error;
pub use mir::*;

pub fn dump(mir: Mir) {
    println!("{mir:#?}");
}
