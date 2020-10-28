use mock_method::MockMethod;

pub struct MockTrait<'a> {
    item_trait: &'a syn::ItemTrait,
    is_sized: bool,
    methods: Vec<MockMethod<'a>>,
}

impl<'a> MockTrait<'a> {
    fn is_sized(bound: &syn::TraitBound) -> Option<bool> {
	bound.path.segments.last()
	    .and_then(|v| { if v.ident == "Sized" { Some(true) } else { None }})
    }
    
    fn det_impls_size(trait_block: &'a syn::ItemTrait) -> bool {
	let num_sized = trait_block.supertraits.iter()
	    .filter_map(|ref x| {
		match x {
		    syn::TypeParamBound::Trait(ref y) => MockTrait::is_sized(y),
		    _ => None
		}
	    })
	    .count();

	num_sized > 0
    }

    fn generate_methods(trait_block: &'a syn::ItemTrait) -> Vec<MockMethod> {
	trait_block.items
	    .iter()
	    .map(|x| {
		match x {
		    syn::TraitItem::Method(f) => Some(f),
		    _ => None
		}
	    })
	    .filter(|x| x.is_some())
	    .map(|x| MockMethod::with_method(x.unwrap())) // Will be 'SOME' due to the previous map
	    .collect::<Vec<MockMethod>>()
    }

    pub fn with_trait(input: &'a syn::ItemTrait) -> MockTrait<'a> {
	let is_sized = MockTrait::det_impls_size(&input);
	let methods = MockTrait::generate_methods(&input);
	
	MockTrait {
	    item_trait: input,
	    is_sized: is_sized,
	    methods: vec![],
	}
    }

}


