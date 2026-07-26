//! This library provides primitive traits to serialize and deserialize
//! Linked-Data types. It is shipped with derive macros (using the `derive`
//! feature) that can automatically implement those primitives for you.
//!
//! A value is described as a set of RDF quads: [`LinkedDataSubject`] says
//! which properties a subject carries, [`LinkedDataPredicateObjects`] which
//! objects a predicate points at, and [`LinkedDataGraph`] which graph they
//! belong to. The derive macros implement all of them from `#[ld(...)]`
//! attributes.
//!
//! # Example
//!
//! ```
//! use iri_rs::IriBuf;
//! use ld_core::rdfx::{RdfDisplay, generator};
//! use ld_core::{Deserialize, Serialize, to_quads};
//!
//! #[derive(Serialize, Deserialize)]
//! #[ld(prefix("ex" = "http://example.org/"))]
//! struct Foo {
//!     #[ld(id)]
//!     id: IriBuf,
//!
//!     #[ld("ex:name")]
//!     name: String,
//!
//!     #[ld("ex:email")]
//!     email: String,
//! }
//!
//! let value = Foo {
//!     id: IriBuf::new("http://example.org/JohnSmith".to_owned()).unwrap(),
//!     name: "John Smith".to_owned(),
//!     email: "john.smith@example.org".to_owned(),
//! };
//!
//! let quads = to_quads(generator::Blank::new(), &value).unwrap();
//! let output: Vec<String> = quads.iter().map(|q| format!("{} .", q.rdf_display())).collect();
//!
//! assert!(output.contains(
//!     &r#"<http://example.org/JohnSmith> <http://example.org/name> "John Smith" ."#.to_owned()
//! ));
//! ```
//!
//! # Feature flags
//!
//! - `derive` (default) — the [`Serialize`] and [`Deserialize`] derive macros.
//! - `serde` (default) — `serde` interop, including the [`json_literal!`]
//!   macro that maps a `serde` type onto an `rdf:JSON` literal.
use educe::Educe;
use iri_rs::{Iri, IriBuf};
#[cfg(feature = "derive")]
pub use ld_core_derive::{Deserialize, Serialize};
use rdfx::{
	Interpretation, dataset::PatternMatchingDataset, interpretation::ReverseInterpretation,
};

#[doc(hidden)]
pub use iri_rs;

#[doc(hidden)]
pub use rdfx;

#[doc(hidden)]
pub use xsd_rs;

#[doc(hidden)]
pub use jstrict;

mod anonymous;
mod datatypes;
mod graph;
mod r#impl;
mod macros;
mod predicate;
mod quads;
mod rdf;
mod reference;
mod resource;
mod subject;

pub use anonymous::*;
pub use graph::*;
pub use predicate::*;
pub use quads::{
	IntoQuadsError, to_interpreted_graph_quads, to_interpreted_quads, to_interpreted_subject_quads,
	to_lexical_quads, to_lexical_quads_with, to_lexical_subject_quads,
	to_lexical_subject_quads_with, to_quads, to_quads_with,
};
pub use rdf::*;
pub use reference::*;
pub use resource::*;
pub use subject::*;

#[derive(Debug, thiserror::Error)]
/// Error raised while deserializing a value from Linked Data.
pub enum FromLinkedDataError {
	/// Resource has no IRI representation.
	#[error("expected IRI")]
	ExpectedIri(ContextIris),

	#[error("unsupported IRI `{found}`")]
	/// Resource is identified by an IRI the type does not accept.
	UnsupportedIri {
		/// Error context.
		context: ContextIris,

		/// Unsupported IRI.
		found: IriBuf,

		/// Optional hint listing the supported IRIs.
		supported: Option<Vec<IriBuf>>,
	},

	/// Resource has no literal representation.
	#[error("expected literal")]
	ExpectedLiteral(ContextIris),

	/// Resource has literal representations, but none of the expected type.
	///
	/// The type IRIs are boxed: this is the only variant carrying two of them,
	/// and inline they would make it the largest variant by a wide margin, which
	/// sets the size of the whole enum.
	#[error("literal type mismatch")]
	LiteralTypeMismatch {
		/// Error context.
		context: ContextIris,
		/// Expected literal type, if the type accepts only one.
		expected: Option<Box<IriBuf>>,
		/// Literal type actually found.
		found: Box<IriBuf>,
	},

	/// Resource has a literal representation of the correct type, but the
	/// lexical value could not be successfully parsed.
	#[error("invalid literal")]
	InvalidLiteral(ContextIris),

	/// Missing required value.
	#[error("missing required value")]
	MissingRequiredValue(ContextIris),

	/// Too many values.
	#[error("too many values")]
	TooManyValues(ContextIris),

	/// Generic error for invalid subjects.
	#[error("invalid subject")]
	InvalidSubject {
		/// Error context.
		context: ContextIris,
		/// Subject IRI, if it has one.
		subject: Option<IriBuf>,
	},
}

impl FromLinkedDataError {
	/// Returns the context in which this error was raised.
	pub fn context(&self) -> &ContextIris {
		match self {
			Self::ExpectedIri(c) => c,
			Self::UnsupportedIri { context, .. } => context,
			Self::ExpectedLiteral(c) => c,
			Self::LiteralTypeMismatch { context, .. } => context,
			Self::InvalidLiteral(c) => c,
			Self::MissingRequiredValue(c) => c,
			Self::TooManyValues(c) => c,
			Self::InvalidSubject { context, .. } => context,
		}
	}
}

/// Linked-Data type.
///
/// A Linked-Data type represents an RDF dataset which can be visited using the
/// [`visit`](Self::visit) method.
pub trait LinkedData<I: Interpretation = ()> {
	/// Visit the RDF dataset represented by this type.
	fn visit<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: Visitor<I>;
}

impl<I: Interpretation, T: ?Sized + LinkedData<I>> LinkedData<I> for &T {
	fn visit<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: Visitor<I>,
	{
		T::visit(self, visitor)
	}
}

impl<I: Interpretation, T: ?Sized + LinkedData<I>> LinkedData<I> for Box<T> {
	fn visit<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: Visitor<I>,
	{
		T::visit(self, visitor)
	}
}

impl<I: Interpretation> LinkedData<I> for IriBuf {
	fn visit<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: Visitor<I>,
	{
		visitor.end()
	}
}

/// RDF dataset visitor.
pub trait Visitor<I: Interpretation = ()> {
	/// Type of the value returned by the visitor when the dataset has been
	/// entirely visited.
	type Ok;

	/// Error type.
	type Error;

	/// Visits the default graph of the dataset.
	fn default_graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataGraph<I>;

	/// Visits a named graph of the dataset.
	fn named_graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + LinkedDataGraph<I>;

	/// Ends the dataset visit.
	fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Any mutable reference to a visitor is itself a visitor.
impl<I: Interpretation, S: Visitor<I>> Visitor<I> for &mut S {
	type Ok = ();
	type Error = S::Error;

	fn default_graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataGraph<I>,
	{
		S::default_graph(self, value)
	}

	fn named_graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + LinkedDataGraph<I>,
	{
		S::named_graph(self, value)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(())
	}
}

#[derive(Educe)]
#[educe(Debug(bound = "I::Resource: core::fmt::Debug"), Clone, Copy)]
/// Reference to a resource, either interpreted or lexical.
pub enum ResourceOrIriRef<'a, I: Interpretation> {
	/// A resource of the interpretation.
	Resource(&'a I::Resource),
	/// An IRI, outside of any interpretation.
	Iri(Iri<&'a str>),
	/// An anonymous resource, with no identifier.
	Anonymous,
}

impl<'a, I: Interpretation> ResourceOrIriRef<'a, I> {
	/// Resolves this reference into an IRI, if it has one.
	pub fn into_iri(self, interpretation: &I) -> Option<IriBuf>
	where
		I: ReverseInterpretation,
	{
		match self {
			Self::Resource(r) => interpretation.iris_of(r).next().map(|i| i.into_owned()),
			Self::Iri(i) => Some(i.into()),
			Self::Anonymous => None,
		}
	}
}

#[derive(Educe)]
#[educe(Debug(bound = "I::Resource: core::fmt::Debug"), Clone, Copy)]
/// Position a value occupies while being visited, used to report errors.
#[derive(Default)]
pub enum Context<'a, I: Interpretation> {
	/// The value is a subject.
	#[default]
	Subject,
	/// The value is a predicate of the given subject.
	Predicate {
		/// Subject the predicate belongs to.
		subject: ResourceOrIriRef<'a, I>,
	},
	/// The value is an object of the given subject and predicate.
	Object {
		/// Subject the object belongs to.
		subject: ResourceOrIriRef<'a, I>,
		/// Predicate the object belongs to.
		predicate: ResourceOrIriRef<'a, I>,
	},
}

impl<'a, I: Interpretation> Context<'a, I> {
	/// Moves this context into the predicate position of `subject`.
	pub fn with_subject(self, subject: &'a I::Resource) -> Self {
		Self::Predicate {
			subject: ResourceOrIriRef::Resource(subject),
		}
	}

	/// Moves this context into the object position of `predicate`.
	pub fn with_predicate(self, predicate: &'a I::Resource) -> Self {
		match self {
			Self::Predicate { subject } => Self::Object {
				subject,
				predicate: ResourceOrIriRef::Resource(predicate),
			},
			_ => Self::Subject,
		}
	}

	/// Moves this context into the object position of the `predicate` IRI.
	pub fn with_predicate_iri(self, predicate: Iri<&'a str>) -> Self {
		match self {
			Self::Predicate { subject } => Self::Object {
				subject,
				predicate: ResourceOrIriRef::Iri(predicate),
			},
			_ => Self::Subject,
		}
	}

	/// Moves this context into the object position of an anonymous predicate.
	pub fn with_anonymous_predicate(self) -> Self {
		match self {
			Self::Predicate { subject } => Self::Object {
				subject,
				predicate: ResourceOrIriRef::Anonymous,
			},
			_ => Self::Subject,
		}
	}

	/// Resolves the resources of this context into IRIs.
	pub fn into_iris(self, interpretation: &I) -> ContextIris
	where
		I: ReverseInterpretation,
	{
		match self {
			Self::Subject => ContextIris::Subject,
			Self::Predicate { subject } => ContextIris::Predicate {
				subject: subject.into_iri(interpretation).map(Box::new),
			},
			Self::Object { subject, predicate } => ContextIris::Object {
				subject: subject.into_iri(interpretation).map(Box::new),
				predicate: predicate.into_iri(interpretation).map(Box::new),
			},
		}
	}
}

#[derive(Debug, Clone)]
/// Error context with its resources resolved into IRIs.
///
/// The IRIs are boxed because this type is carried by every
/// [`FromLinkedDataError`] variant, and therefore sits in the `Err` slot of
/// every deserialization `Result`. An inline [`IriBuf`] is 56 bytes, which
/// would make the error — and so every `Result` returned on the success path
/// too — several times larger than the values being deserialized.
pub enum ContextIris {
	/// The value is a subject.
	Subject,
	/// The value is a predicate of the given subject.
	Predicate {
		/// Subject IRI, if it has one.
		subject: Option<Box<IriBuf>>,
	},
	/// The value is an object of the given subject and predicate.
	Object {
		/// Subject IRI, if it has one.
		subject: Option<Box<IriBuf>>,
		/// Predicate IRI, if it has one.
		predicate: Option<Box<IriBuf>>,
	},
}

/// Type that can be deserialized from an RDF dataset.
pub trait LinkedDataDeserialize<I: Interpretation>: Sized
where
	I::Resource: rdfx::Resource,
{
	/// Deserializes a value from `dataset`, reporting errors against
	/// `context`.
	fn deserialize_dataset_in(
		interpretation: &I,
		dataset: &(
		     impl rdfx::dataset::TraversableDataset<Subject = I::Resource>
		     + PatternMatchingDataset<Subject = I::Resource>
		 ),
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>;

	/// Deserializes a value from `dataset`.
	fn deserialize_dataset(
		interpretation: &I,
		dataset: &(
		     impl rdfx::dataset::TraversableDataset<Subject = I::Resource>
		     + PatternMatchingDataset<Subject = I::Resource>
		 ),
	) -> Result<Self, FromLinkedDataError> {
		Self::deserialize_dataset_in(interpretation, dataset, Context::default())
	}
}
