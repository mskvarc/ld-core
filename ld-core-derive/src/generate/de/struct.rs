use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::generate::{TypeAttributes, extend_generics};

use super::{Error, FieldsDeserialization, generate_fields};

pub fn generate(
	attrs: &TypeAttributes,
	ident: Ident,
	generics: syn::Generics,
	s: syn::DataStruct,
) -> Result<TokenStream, Error> {
	let FieldsDeserialization {
		deserialize_fields,
		constructor,
		mut interpretation_bounds,
		bounds,
	} = generate_fields(attrs, s.fields)?;

	interpretation_bounds.reverse = true;

	let ld_generics = extend_generics(&generics, interpretation_bounds, bounds);
	let (_, ty_generics, _) = generics.split_for_impl();
	let (impl_generics, _, where_clause) = ld_generics.split_for_impl();

	Ok(quote! {
		impl #impl_generics ::ld_core::LinkedDataDeserializeSubject<I_> for #ident #ty_generics #where_clause {
			fn deserialize_subject_in<D_>(
				interpretation_: &I_,
				dataset_: &D_,
				graph_: Option<&I_::Resource>,
				resource_: &I_::Resource,
				context_: ::ld_core::Context<I_>
			) -> Result<Self, ::ld_core::FromLinkedDataError>
			where
				D_: ::ld_core::rdfx::dataset::PatternMatchingDataset<Subject = I_::Resource>
			{
				let context_ = context_.with_subject(resource_);
				#(#deserialize_fields)*
				Ok(Self #constructor)
			}
		}

		impl #impl_generics ::ld_core::LinkedDataDeserializePredicateObjects<I_> for #ident #ty_generics #where_clause {
			fn deserialize_objects_in<'de_, D_>(
				interpretation: &I_,
				dataset: &D_,
				graph: Option<&I_::Resource>,
				objects: impl IntoIterator<Item = &'de_ I_::Resource>,
				context: ::ld_core::Context<I_>
			) -> Result<Self, ::ld_core::FromLinkedDataError>
			where
				I_::Resource: 'de_,
				D_: ::ld_core::rdfx::dataset::PatternMatchingDataset<Subject = I_::Resource>
			{
				let mut objects = objects.into_iter();

				match objects.next() {
					Some(object) => {
						let value = <Self as ::ld_core::LinkedDataDeserializeSubject<I_>>::deserialize_subject_in(
							interpretation,
							dataset,
							graph,
							object,
							context
						)?;

						if objects.next().is_some() {
							Err(::ld_core::FromLinkedDataError::TooManyValues(
								context.into_iris(interpretation)
							))
						} else {
							Ok(value)
						}
					}
					None => {
						Err(::ld_core::FromLinkedDataError::MissingRequiredValue(
							context.into_iris(interpretation)
						))
					}
				}
			}
		}
	})
}
