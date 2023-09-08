mod parser;

macro_rules! valid {
    ($class: ident, [$($input: expr,)+]) => {{
        $(
            match $crate::idl_parser::IDLParser::parse($crate::Rule::$class, $input) {
                Ok(_) => {},
                Err(e) => panic!("Expected success, but expr: {:?} generated: {:?}", $input, e),
            }
        )+
    }};
}

macro_rules! invalid {
    ($class: ident, [$($input: expr,)+]) => {{
        $(
            match $crate::idl_parser::IDLParser::parse($crate::Rule::$class, $input) {
                Err(_) => {},
                Ok(o) => panic!("Expected failure, but expr: {:?} generated: {:?}", $input, o),
            }
        )+
    }};
}

pub(super) use {invalid, valid};
