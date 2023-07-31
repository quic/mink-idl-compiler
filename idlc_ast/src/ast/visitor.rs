use super::{Interface, StructField};

pub trait Visitor {
    fn visit_struct_field(&mut self, field: &StructField) -> String;
    fn visit_struct_prefix(&mut self, ident: &str) -> String;
    fn visit_struct_suffix(&mut self, ident: &str) -> String;
    fn struct_field_seperator(&self) -> &'static str;

    fn visit_struct(&mut self, ident: &str, fields: &[StructField]) -> String {
        let mut init = self.visit_struct_prefix(ident);
        for field in fields {
            init += &self.visit_struct_field(field);
            init += self.struct_field_seperator()
        }
        init + &self.visit_struct_suffix(ident)
    }
    fn visit_include(&mut self, include: &str) -> String;
    fn visit_caller(&mut self, interface: &Interface) -> String;
    fn visit_callee(&mut self, interface: &Interface) -> String;
}
