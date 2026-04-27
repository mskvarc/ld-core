use std::collections::HashMap;

use proc_macro2::TokenStream;

use quote::{format_ident, quote};
use syn::{DeriveInput, spanned::Spanned};

use super::{Error, InterpretationBounds, TypeAttributes, read_field_attributes, read_type_attributes};

mod r#enum;
mod r#struct;

pub fn subject(input: DeriveInput) -> Result<TokenStream, Error> {
	let attrs = read_type_attributes(input.attrs)?;
	match input.data {
		syn::Data::Struct(s) => r#struct::generate(&attrs, input.ident, input.generics, s),
		syn::Data::Enum(e) => r#enum::generate(&attrs, input.ident, input.generics, e),
		syn::Data::Union(u) => Err(Error::UnionType(u.union_token.span())),
	}
}

struct FieldsDeserialization {
	deserialize_fields: Vec<TokenStream>,
	constructor: TokenStream,
	interpretation_bounds: InterpretationBounds,
	bounds: Vec<syn::WherePredicate>,
}

fn generate_fields(
	attrs: &TypeAttributes,
	fields: syn::Fields,
) -> Result<FieldsDeserialization, Error> {
	let mut interpretation_bounds = InterpretationBounds::default();
	let mut bounds = Vec::new();
	let mut deserialize_fields = Vec::with_capacity(fields.len());
	let constructor = match fields {
		syn::Fields::Unit => quote!(),
		syn::Fields::Unnamed(fields) => {
			let mut fields_constructors = Vec::with_capacity(fields.unnamed.len());
			for (i, f) in fields.unnamed.into_iter().enumerate() {
				let ident = format_ident!("a{i}");
				match generate_field(
					&attrs.prefixes,
					f.attrs,
					&f.ty,
					&mut interpretation_bounds,
					&mut bounds,
				)? {
					Some(deserialize_field) => {
						deserialize_fields.push(quote! {
							let #ident = #deserialize_field ;
						});

						fields_constructors.push(quote!(#ident));
					}
					None => {
						let ty = &f.ty;
						fields_constructors
							.push(quote!(<#ty as ::std::default::Default>::default()));
					}
				}
			}

			quote!(( #(#fields_constructors),* ))
		}
		syn::Fields::Named(fields) => {
			let mut fields_constructors = Vec::with_capacity(fields.named.len());
			for f in fields.named {
				let ident = f.ident;

				match generate_field(
					&attrs.prefixes,
					f.attrs,
					&f.ty,
					&mut interpretation_bounds,
					&mut bounds,
				)? {
					Some(deserialize_field) => {
						deserialize_fields.push(quote! {
							let #ident = #deserialize_field ;
						});

						fields_constructors.push(quote!(#ident));
					}
					None => {
						let ty = &f.ty;
						fields_constructors
							.push(quote!(#ident: <#ty as ::std::default::Default>::default()));
					}
				}
			}

			quote!({ #(#fields_constructors),* })
		}
	};

	Ok(FieldsDeserialization {
		deserialize_fields,
		constructor,
		interpretation_bounds,
		bounds,
	})
}

fn generate_field(
	prefixes: &HashMap<String, String>,
	attrs: Vec<syn::Attribute>,
	ty: &syn::Type,
	interpretation_bounds: &mut InterpretationBounds,
	bounds: &mut Vec<syn::WherePredicate>,
) -> Result<Option<TokenStream>, Error> {
	let attrs = read_field_attributes(attrs)?;

	match attrs.iri {
		Some(compact_iri) => {
			let iri = compact_iri.expand(prefixes)?.to_string();
			interpretation_bounds.reverse = true;

			bounds.push(
				syn::parse2(quote!(#ty: ::ld_core::LinkedDataDeserializePredicateObjects<I_>))
					.unwrap(),
			);

			Ok(Some(quote! {
				{
					let _iri_buf = ::ld_core::iri_rs::IriBuf::new(#iri.to_owned()).unwrap();
					match ::ld_core::rdf_rs::Interpretation::iri(interpretation_, _iri_buf.as_ref()) {
						Some(predicate_) => {
							let context_ = context_.with_predicate(&predicate_);
							::ld_core::LinkedDataDeserializePredicateObjects::deserialize_objects_in(
								interpretation_,
								dataset_,
								graph_,
								::ld_core::rdf_rs::dataset::PatternMatchingDataset::quad_objects(dataset_, graph_, resource_, &predicate_),
								context_
							)?
						}
						None => {
							let context_ = context_.with_predicate_iri(_iri_buf.as_ref());
							::ld_core::LinkedDataDeserializePredicateObjects::deserialize_objects_in(
								interpretation_,
								dataset_,
								graph_,
								[],
								context_
							)?
						}
					}
				}
			}))
		}
		None => {
			if attrs.is_id || attrs.flatten {
				bounds.push(
					syn::parse2(quote!(#ty: ::ld_core::LinkedDataDeserializeSubject<I_>))
						.unwrap(),
				);

				Ok(Some(quote! {
					::ld_core::LinkedDataDeserializeSubject::deserialize_subject_in(
						interpretation_,
						dataset_,
						graph_,
						resource_,
						context_
					)?
				}))
			} else if attrs.ignore {
				Ok(None)
			} else {
				panic!()
			}
		}
	}
}
