use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::generate::{InterpretationBounds, RDF_TYPE, TypeAttributes, extend_generics};

use super::{Error, variant_compound_fields};

pub fn generate(
	attrs: &TypeAttributes,
	ident: Ident,
	generics: syn::Generics,
	s: syn::DataStruct,
) -> Result<TokenStream, Error> {
	let fields = variant_compound_fields(
		attrs,
		s.fields,
		|f| quote!(self.#f),
		|i| {
			let index = syn::Index {
				index: i,
				span: Span::call_site(),
			};

			quote!(self.#index)
		},
		|t| quote!(&#t),
	)?;

	let mut bounds: Vec<syn::WherePredicate> = fields.visit.bounds;

	let visit_type = attrs
		.type_
		.as_ref()
		.map(|ty| {
			let iri = ty.expand(&attrs.prefixes)?.to_string();
			let rdf_type = RDF_TYPE;

			Ok(quote! {
				visitor.predicate(
					&::ld_core::iri_rs::IriBuf::new(#rdf_type.to_owned()).unwrap(),
					&::ld_core::iri_rs::IriBuf::new(#iri.to_owned()).unwrap()
				)?;
			})
		})
		.transpose()?;

	let visit = fields.visit.body;

	let term = match fields.id_field {
		Some((field_access, ty)) => {
			bounds.push(
				syn::parse2(quote! {
					#ty: ::ld_core::LinkedDataResource<I_>
				})
				.unwrap(),
			);

			quote! {
				#field_access.interpretation(interpretation)
			}
		}
		None => quote! {
			::ld_core::ResourceInterpretation::Uninterpreted(None)
		},
	};

	let ld_generics = extend_generics(&generics, InterpretationBounds::default(), bounds);
	let (_, ty_generics, _) = generics.split_for_impl();
	let (impl_generics, _, where_clause) = ld_generics.split_for_impl();

	Ok(quote! {
		impl #impl_generics ::ld_core::LinkedDataResource<I_> for #ident #ty_generics #where_clause {
			fn interpretation(
				&self,
				interpretation: &mut I_
			) -> ::ld_core::ResourceInterpretation<'_, I_> {
				#term
			}
		}

		impl #impl_generics ::ld_core::LinkedDataSubject<I_> for #ident #ty_generics #where_clause {
			fn visit_subject<S_>(&self, mut visitor: S_) -> Result<S_::Ok, S_::Error>
			where
				S_: ::ld_core::SubjectVisitor<I_>
			{
				#visit_type
				#visit
			}
		}

		impl #impl_generics ::ld_core::LinkedDataPredicateObjects<I_> for #ident #ty_generics #where_clause {
			fn visit_objects<S_>(&self, mut visitor: S_) -> Result<S_::Ok, S_::Error>
			where
				S_: ::ld_core::PredicateObjectsVisitor<I_>
			{
				visitor.object(self)?;
				visitor.end()
			}
		}

		impl #impl_generics ::ld_core::LinkedDataGraph<I_> for #ident #ty_generics #where_clause {
			fn visit_graph<S_>(&self, mut visitor: S_) -> Result<S_::Ok, S_::Error>
			where
				S_: ::ld_core::GraphVisitor<I_>
			{
				visitor.subject(self)?;
				visitor.end()
			}
		}

		impl #impl_generics ::ld_core::LinkedData<I_> for #ident #ty_generics #where_clause {
			fn visit<S_>(&self, mut visitor: S_) -> Result<S_::Ok, S_::Error>
			where
				S_: ::ld_core::Visitor<I_>
			{
				visitor.default_graph(self)?;
				visitor.end()
			}
		}
	})
}
