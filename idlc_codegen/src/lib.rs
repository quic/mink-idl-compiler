pub mod counts;
pub mod functions;
pub mod serialization;
use std::path::PathBuf;

// FIXME: This probably doesn't even need to contain the first part of the tuple
// if all codegen units are guaranteed to return exactly 1 output per file. In
// which case we can use a transform function in the [`Generator`] to transform
// input IDL name to output
pub type Descriptor = Vec<(PathBuf, String)>;

/// Codegenerator backends for [`idlc_mir`]
pub trait Generator {
    /// Generates the backend language based on input IDL
    fn generate(mir: &idlc_mir::Mir) -> Descriptor;
}
