pub struct MockMethod<'a> {
    method: &'a syn::TraitItemMethod,
    ownership: InstanceMethodOwnership,
}

enum InstanceMethodOwnership {
    SelfOwnership,
    SelfReference,
    StaticMethod
}

impl<'a> MockMethod<'a> {
    fn det_ownership(method_item: &'a syn::TraitItemMethod) -> InstanceMethodOwnership {
	method_item.sig.inputs
	    .iter()
	    .nth(0)
	    .map_or(InstanceMethodOwnership::StaticMethod,
		    |x| {
			match x {
			    syn::FnArg::Receiver(aself) => {
				if aself.reference.is_some() {
				    InstanceMethodOwnership::SelfReference
				} else {
				    InstanceMethodOwnership::SelfOwnership
				}
			    },
			    _ => InstanceMethodOwnership::StaticMethod 
			}
	    })
    }
    
    pub fn with_method(method_item: &'a syn::TraitItemMethod) -> MockMethod<'a> {
	let method_ownership = MockMethod::det_ownership(&method_item);

	MockMethod {
	    method: method_item,
	    ownership: method_ownership,
	}
    }
}
