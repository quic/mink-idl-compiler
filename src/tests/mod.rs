mod parser;

macro_rules! valid {
    ($class: ident, [$($input: expr,)+]) => {{
        $(
            #[allow(unused)]
            use crate::*;
            match IDLParser::parse(Rule::$class, $input) {
                Ok(_) => {},
                Err(e) => panic!("Expected success, but expr: {:?} generated: {:?}", $input, e),
            }
        )+
    }};
}

macro_rules! invalid {
    ($class: ident, [$($input: expr,)+]) => {{
        $(
            #[allow(unused)]
            use crate::*;
            match IDLParser::parse(Rule::$class, $input) {
                Err(_) => {},
                Ok(o) => panic!("Expected failure, but expr: {:?} generated: {:?}", $input, o),
            }
        )+
    }};
}

pub(crate) use {invalid, valid};
