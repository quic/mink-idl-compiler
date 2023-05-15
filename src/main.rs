#[cfg(test)]
mod tests;

#[derive(pest_derive::Parser)]
#[grammar = "../grammar/idl.pest"]
pub struct IDLParser;

fn main() {
    println!("Hello, world!");
}
