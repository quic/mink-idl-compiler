use idlc_mir::Primitive;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pair {
    ident: idlc_mir::Ident,
    ty: Primitive,
    nth_param: usize,
}

impl Pair {
    pub fn new(ident: &idlc_mir::Ident, ty: Primitive, nth_param: usize) -> Self {
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
    fn visit_input_primitive(&mut self, ident: &idlc_mir::Ident, ty: &Primitive) {
        let nth_param = self.inputs.len();
        self.inputs.push(Pair::new(ident, *ty, nth_param));
        self.input_size += ty.size();
    }

    fn visit_output_primitive(&mut self, ident: &idlc_mir::Ident, ty: &Primitive) {
        let nth_param = self.outputs.len();
        self.outputs.push(Pair::new(ident, *ty, nth_param));
        self.output_size += ty.size();
    }
}

impl PackedPrimitives {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();
        super::functions::visit_params(function, &mut me);

        me.inputs.sort_by(|a, b| b.cmp(a));
        me.outputs.sort_by(|a, b| b.cmp(a));

        me
    }

    pub fn inputs_by_idents(&self) -> impl ExactSizeIterator<Item = (&idlc_mir::Ident, Primitive)> {
        self.inputs.iter().map(|pair| (&pair.ident, pair.ty))
    }

    pub fn inputs_by_index(&self) -> impl ExactSizeIterator<Item = (usize, Primitive)> + '_ {
        self.inputs.iter().map(|pair| (pair.nth_param, pair.ty))
    }

    pub fn input_types(&self) -> impl ExactSizeIterator<Item = Primitive> + '_ {
        self.inputs.iter().map(|pair| pair.ty)
    }

    pub fn n_inputs(&self) -> usize {
        self.inputs.len()
    }

    pub const fn packed_input_size(&self) -> usize {
        self.input_size
    }

    pub fn outputs_by_idents(
        &self,
    ) -> impl ExactSizeIterator<Item = (&idlc_mir::Ident, Primitive)> {
        self.outputs.iter().map(|pair| (&pair.ident, pair.ty))
    }

    pub fn outputs_by_index(&self) -> impl ExactSizeIterator<Item = (usize, Primitive)> + '_ {
        self.outputs.iter().map(|pair| (pair.nth_param, pair.ty))
    }

    pub fn output_types(&self) -> impl ExactSizeIterator<Item = Primitive> + '_ {
        self.outputs.iter().map(|pair| pair.ty)
    }

    pub fn n_outputs(&self) -> usize {
        self.outputs.len()
    }

    pub const fn packed_output_size(&self) -> usize {
        self.output_size
    }
}
