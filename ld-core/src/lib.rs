//! This library provides primitive traits to serialize and deserialize
//! Linked-Data types. It is shipped with derive macros (using the `derive`
//! feature) that can automatically implement those primitives for you.
//!
//! # Example
//!
//! ```ignore
//! use iri_rs::{IriBuf, iri};
//!
//! #[derive(ld_core::Serialize, ld_core::Deserialize)]
//! #[ld(prefix("ex" = "http://example.org/"))]
//! struct Foo {
//!   #[ld(id)]
//!   id: IriBuf,
//!
//!   #[ld("ex:name")]
//!   name: String,
//!
//!   #[ld("ex:email")]
//!   email: String
//! }
//! ```
use educe::Educe;
use iri_rs::{Iri, IriBuf};
#[cfg(feature = "derive")]
pub use ld_core_derive::{Deserialize, Serialize};
use rdf_rs::{
	Interpretation, dataset::PatternMatchingDataset, interpretation::ReverseInterpretation,
};

#[doc(hidden)]
pub use iri_rs;

#[doc(hidden)]
pub use rdf_rs;

#[doc(hidden)]
pub use xsd_rs;

#[doc(hidden)]
pub use json_syntax;

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
pub enum FromLinkedDataError {
	/// Resource has no IRI representation.
	#[error("expected IRI")]
	ExpectedIri(ContextIris),

	#[error("unsupported IRI `{found}`")]
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
	#[error("literal type mismatch")]
	LiteralTypeMismatch {
		context: ContextIris,
		expected: Option<IriBuf>,
		found: IriBuf,
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
		context: ContextIris,
		subject: Option<IriBuf>,
	},
}

impl FromLinkedDataError {
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

impl<'a, I: Interpretation, T: ?Sized + LinkedData<I>> LinkedData<I> for &'a T {
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
impl<'s, I: Interpretation, S: Visitor<I>> Visitor<I> for &'s mut S {
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
pub enum ResourceOrIriRef<'a, I: Interpretation> {
	Resource(&'a I::Resource),
	Iri(Iri<&'a str>),
	Anonymous,
}

impl<'a, I: Interpretation> ResourceOrIriRef<'a, I> {
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
pub enum Context<'a, I: Interpretation> {
	Subject,
	Predicate {
		subject: ResourceOrIriRef<'a, I>,
	},
	Object {
		subject: ResourceOrIriRef<'a, I>,
		predicate: ResourceOrIriRef<'a, I>,
	},
}

impl<'a, I: Interpretation> Context<'a, I> {
	pub fn with_subject(self, subject: &'a I::Resource) -> Self {
		Self::Predicate {
			subject: ResourceOrIriRef::Resource(subject),
		}
	}

	pub fn with_predicate(self, predicate: &'a I::Resource) -> Self {
		match self {
			Self::Predicate { subject } => Self::Object {
				subject,
				predicate: ResourceOrIriRef::Resource(predicate),
			},
			_ => Self::Subject,
		}
	}

	pub fn with_predicate_iri(self, predicate: Iri<&'a str>) -> Self {
		match self {
			Self::Predicate { subject } => Self::Object {
				subject,
				predicate: ResourceOrIriRef::Iri(predicate),
			},
			_ => Self::Subject,
		}
	}

	pub fn with_anonymous_predicate(self) -> Self {
		match self {
			Self::Predicate { subject } => Self::Object {
				subject,
				predicate: ResourceOrIriRef::Anonymous,
			},
			_ => Self::Subject,
		}
	}

	pub fn into_iris(self, interpretation: &I) -> ContextIris
	where
		I: ReverseInterpretation,
	{
		match self {
			Self::Subject => ContextIris::Subject,
			Self::Predicate { subject } => ContextIris::Predicate {
				subject: subject.into_iri(interpretation),
			},
			Self::Object { subject, predicate } => ContextIris::Object {
				subject: subject.into_iri(interpretation),
				predicate: predicate.into_iri(interpretation),
			},
		}
	}
}

#[derive(Debug, Clone)]
pub enum ContextIris {
	Subject,
	Predicate {
		subject: Option<IriBuf>,
	},
	Object {
		subject: Option<IriBuf>,
		predicate: Option<IriBuf>,
	},
}

impl<'a, I: Interpretation> Default for Context<'a, I> {
	fn default() -> Self {
		Self::Subject
	}
}

pub trait LinkedDataDeserialize<I: Interpretation>: Sized
where
	I::Resource: rdf_rs::Resource,
{
	fn deserialize_dataset_in(
		interpretation: &I,
		dataset: &(
		     impl rdf_rs::dataset::TraversableDataset<Subject = I::Resource>
		     + PatternMatchingDataset<Subject = I::Resource>
		 ),
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>;

	fn deserialize_dataset(
		interpretation: &I,
		dataset: &(
		     impl rdf_rs::dataset::TraversableDataset<Subject = I::Resource>
		     + PatternMatchingDataset<Subject = I::Resource>
		 ),
	) -> Result<Self, FromLinkedDataError> {
		Self::deserialize_dataset_in(interpretation, dataset, Context::default())
	}
}
