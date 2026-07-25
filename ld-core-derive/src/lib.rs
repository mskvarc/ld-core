//! Derive macros for the `ld-core` library. Not meant to be used directly:
//! `ld-core` re-exports them behind its `derive` feature.
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use syn::DeriveInput;

mod generate;
pub(crate) mod utils;

/// Derives the `LinkedData` serialization traits from `#[ld(...)]`
/// attributes.
///
/// See the `ld-core` documentation for the attribute vocabulary.
#[proc_macro_derive(Serialize, attributes(ld))]
#[proc_macro_error]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as DeriveInput);
	let mut output = proc_macro2::TokenStream::new();

	match generate::ser::subject(input) {
		Ok(tokens) => output.extend(tokens),
		Err(e) => {
			abort!(e.span(), e)
		}
	}

	output.into()
}

/// Derives the `FromLinkedData` deserialization traits from `#[ld(...)]`
/// attributes.
///
/// See the `ld-core` documentation for the attribute vocabulary.
#[proc_macro_derive(Deserialize, attributes(ld))]
#[proc_macro_error]
pub fn derive_deserialize(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as DeriveInput);
	let mut output = proc_macro2::TokenStream::new();

	match generate::de::subject(input) {
		Ok(tokens) => output.extend(tokens),
		Err(e) => {
			abort!(e.span(), e)
		}
	}

	output.into()
}
