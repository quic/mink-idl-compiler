#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Primitive(idlc_mir::Primitive),
    SmallStruct(idlc_mir::StructInner),
}
impl Type {
    pub fn size(&self) -> usize {
        match self {
            Self::Primitive(p) => p.size(),
            Self::SmallStruct(s) => s.size(),
        }
    }
}
impl PartialOrd<Self> for Type {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for Type {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.size().cmp(&other.size())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pair {
    ident: idlc_mir::Ident,
    ty: Type,
    nth_param: usize,
}

impl Pair {
    pub fn new(ident: &idlc_mir::Ident, ty: Type, nth_param: usize) -> Self {
        Self {
            ident: ident.clone(),
            ty,
            nth_param,
        }
    }
}

impl PartialOrd for Pair {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pair {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ty.cmp(&other.ty)
    }
}

#[derive(Debug, Default, Clone)]
pub struct PackedPrimitives {
    inputs: Vec<Pair>,
    input_size: usize,

    outputs: Vec<Pair>,
    output_size: usize,
}

impl super::functions::ParameterVisitor for PackedPrimitives {
    fn visit_input_primitive(&mut self, ident: &idlc_mir::Ident, ty: idlc_mir::Primitive) {
        let nth_param = self.inputs.len();
        self.inputs
            .push(Pair::new(ident, Type::Primitive(ty), nth_param));
        self.input_size += ty.size();
    }

    fn visit_input_small_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let nth_param = self.inputs.len();
        self.inputs
            .push(Pair::new(ident, Type::SmallStruct(ty.clone()), nth_param));
        self.input_size += ty.size();
    }

    fn visit_output_primitive(&mut self, ident: &idlc_mir::Ident, ty: idlc_mir::Primitive) {
        let nth_param = self.outputs.len();
        self.outputs
            .push(Pair::new(ident, Type::Primitive(ty), nth_param));
        self.output_size += ty.size();
    }

    fn visit_output_small_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let nth_param = self.outputs.len();
        self.outputs
            .push(Pair::new(ident, Type::SmallStruct(ty.clone()), nth_param));
        self.output_size += ty.size();
    }
}

impl PackedPrimitives {
    #[must_use]
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();
        super::functions::visit_params(function, &mut me);

        me.inputs.sort_by(|a, b| b.cmp(a));
        me.outputs.sort_by(|a, b| b.cmp(a));

        me
    }

    #[must_use]
    pub fn inputs_by_idents(&self) -> impl ExactSizeIterator<Item = (&idlc_mir::Ident, &Type)> {
        self.inputs.iter().map(|pair| (&pair.ident, &pair.ty))
    }

    #[must_use]
    pub fn inputs_by_index(&self) -> impl ExactSizeIterator<Item = (usize, &Type)> {
        self.inputs.iter().map(|pair| (pair.nth_param, &pair.ty))
    }

    #[must_use]
    pub fn input_types(&self) -> impl ExactSizeIterator<Item = &Type> {
        self.inputs.iter().map(|pair| &pair.ty)
    }

    #[must_use]
    pub fn input_idents(&self) -> impl ExactSizeIterator<Item = &idlc_mir::Ident> {
        self.inputs.iter().map(|pair| &pair.ident)
    }

    #[must_use]
    pub fn n_inputs(&self) -> usize {
        self.inputs.len()
    }

    #[must_use]
    pub const fn packed_input_size(&self) -> usize {
        self.input_size
    }

    #[must_use]
    pub fn outputs_by_idents(&self) -> impl ExactSizeIterator<Item = (&idlc_mir::Ident, &Type)> {
        self.outputs.iter().map(|pair| (&pair.ident, &pair.ty))
    }

    #[must_use]
    pub fn outputs_by_index(&self) -> impl ExactSizeIterator<Item = (usize, &Type)> {
        self.outputs.iter().map(|pair| (pair.nth_param, &pair.ty))
    }

    #[must_use]
    pub fn output_types(&self) -> impl ExactSizeIterator<Item = &Type> {
        self.outputs.iter().map(|pair| &pair.ty)
    }

    #[must_use]
    pub fn output_idents(&self) -> impl ExactSizeIterator<Item = &idlc_mir::Ident> {
        self.outputs.iter().map(|pair| &pair.ident)
    }

    #[must_use]
    pub fn n_outputs(&self) -> usize {
        self.outputs.len()
    }

    #[must_use]
    pub const fn packed_output_size(&self) -> usize {
        self.output_size
    }
}
