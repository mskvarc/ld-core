use derive_more::Debug;
use iri_rs::IriBuf;
use rdfx::{
	BlankId, BlankIdBuf, Id, Interpretation, Literal, LocalTerm, Term,
	interpretation::{ReverseInterpretation, ReverseLocalInterpretation},
};
use std::fmt;

use crate::{BorrowedRdfTerm, CowRdfTerm, OwnedRdfTerm};

/// Resource interpretation.
#[derive(Debug)]
#[debug(bound(I::Resource: fmt::Debug))]
pub enum ResourceInterpretation<'a, I: Interpretation> {
	/// Interpreted resource.
	Interpreted(&'a I::Resource),

	/// Uninterpreted resource with the given optional lexical representation.
	///
	/// It can be used to give an actual interpretation to the resource, bound
	/// to its lexical representation.
	Uninterpreted(Option<CowRdfTerm<'a>>),
}

impl<'a, I: Interpretation> ResourceInterpretation<'a, I> {
	/// Returns the interpreted resource, if this interpretation is one.
	pub fn as_interpreted(&self) -> Option<&'a I::Resource> {
		match self {
			Self::Interpreted(i) => Some(i),
			_ => None,
		}
	}

	/// Returns the lexical term, if this interpretation is uninterpreted.
	pub fn as_uninterpreted(&self) -> Option<&CowRdfTerm<'a>> {
		match self {
			Self::Uninterpreted(t) => t.as_ref(),
			_ => None,
		}
	}

	/// Resolves this interpretation into a lexical term, looking the resource
	/// up in `interpretation` when necessary.
	pub fn into_lexical_representation(self, interpretation: &'a I) -> Option<CowRdfTerm<'a>>
	where
		I: ReverseInterpretation + ReverseLocalInterpretation,
	{
		match self {
			Self::Interpreted(r) => {
				if let Some(i) = interpretation.iris_of(r).next() {
					return Some(CowRdfTerm::Owned(OwnedRdfTerm::Iri(i.into_owned())));
				}

				if let Some(l) = interpretation.literals_of(r).next() {
					return Some(CowRdfTerm::Owned(
						OwnedRdfTerm::Literal(
							crate::RdfLiteralRef::Any(&l.value.clone(), l.type_.as_ref())
								.into_owned(),
						)
						.into_self_owned(),
					));
				}

				if let Some(b) = interpretation.blank_ids_of(r).next() {
					return Some(CowRdfTerm::Owned(OwnedRdfTerm::BlankId(b.into_owned())));
				}

				None
			}
			Self::Uninterpreted(t) => t,
		}
	}
}

impl OwnedRdfTerm {
	fn into_self_owned(self) -> Self {
		self
	}
}

/// Type that can have an interpretation bound to the given lifetime.
pub trait LinkedDataResourceRef<'a, I: Interpretation = ()> {
	/// Returns the interpretation of this resource, bound to `'a`.
	fn interpretation_ref(&self, interpretation: &mut I) -> ResourceInterpretation<'a, I>;
}

/// Type that can have an interpretation.
pub trait LinkedDataResource<I: Interpretation = ()> {
	/// Returns the interpretation of this resource, inserting it into
	/// `interpretation` if required.
	fn interpretation(&self, interpretation: &mut I) -> ResourceInterpretation<'_, I>;

	/// Returns the lexical term representing this resource, if any.
	fn lexical_representation<'a>(&'a self, interpretation: &'a mut I) -> Option<CowRdfTerm<'a>>
	where
		I: ReverseInterpretation + ReverseLocalInterpretation,
	{
		// Cannot use cleanly because of borrow checker; reconstruct.
		match self.interpretation(interpretation) {
			ResourceInterpretation::Interpreted(_) => {
				// Re-call after reborrowing immutably.
				self.interpretation(interpretation)
					.into_lexical_representation(interpretation)
			}
			ResourceInterpretation::Uninterpreted(t) => t,
		}
	}

	/// Returns the interpretation to use when this resource appears as a
	/// reference rather than as a subject with properties.
	fn reference_interpretation(&self, interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		self.interpretation(interpretation)
	}
}

impl<I: Interpretation, T: ?Sized + LinkedDataResource<I>> LinkedDataResource<I> for &T {
	fn interpretation(&self, interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		T::interpretation(self, interpretation)
	}
}

impl<I: Interpretation, T: ?Sized + LinkedDataResource<I>> LinkedDataResource<I> for Box<T> {
	fn interpretation(&self, interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		T::interpretation(self, interpretation)
	}
}

impl<I: Interpretation> LinkedDataResource<I> for () {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		ResourceInterpretation::Uninterpreted(None)
	}
}

/// Anonymous lexical representation.
///
/// This type implements the `LexicalRepresentation` trait, producing a blank
/// node identifier.
pub struct Anonymous;

impl<I: Interpretation> LinkedDataResource<I> for Anonymous {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		ResourceInterpretation::Uninterpreted(None)
	}
}

impl<I: Interpretation> LinkedDataResource<I> for IriBuf {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(BorrowedRdfTerm::Iri(
			self.as_ref(),
		))))
	}
}

impl<I: Interpretation> LinkedDataResource<I> for iri_rs::Iri<&str> {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(BorrowedRdfTerm::Iri(
			*self,
		))))
	}
}

impl<I: Interpretation> LinkedDataResource<I> for BlankId {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(BorrowedRdfTerm::BlankId(
			self,
		))))
	}
}

impl<I: Interpretation> LinkedDataResource<I> for BlankIdBuf {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(BorrowedRdfTerm::BlankId(
			self.as_ref(),
		))))
	}
}

impl<I: Interpretation> LinkedDataResource<I> for Id {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		match self {
			Self::Iri(i) => ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(
				BorrowedRdfTerm::Iri(i.as_ref()),
			))),
			Self::BlankId(b) => ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(
				BorrowedRdfTerm::BlankId(b),
			))),
		}
	}
}

impl<I: Interpretation> LinkedDataResource<I> for Term {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		match self {
			Self::Iri(i) => ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(
				BorrowedRdfTerm::Iri(i.as_ref()),
			))),
			Self::Literal(l) => ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(
				BorrowedRdfTerm::Literal(crate::RdfLiteralRef::Any(&l.value, l.type_.as_ref())),
			))),
		}
	}
}

impl<I: Interpretation> LinkedDataResource<I> for LocalTerm {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		match self {
			Self::BlankId(b) => ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(
				BorrowedRdfTerm::BlankId(b),
			))),
			Self::Named(Term::Iri(i)) => ResourceInterpretation::Uninterpreted(Some(
				CowRdfTerm::Borrowed(BorrowedRdfTerm::Iri(i.as_ref())),
			)),
			Self::Named(Term::Literal(l)) => {
				ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(
					BorrowedRdfTerm::Literal(crate::RdfLiteralRef::Any(&l.value, l.type_.as_ref())),
				)))
			}
			Self::Triple(_) => ResourceInterpretation::Uninterpreted(None),
		}
	}
}

impl<I: Interpretation> LinkedDataResource<I> for Literal {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(BorrowedRdfTerm::Literal(
			crate::RdfLiteralRef::Any(self.value.as_str(), self.type_.as_ref()),
		))))
	}
}

impl<I: Interpretation, T: LinkedDataResource<I>> LinkedDataResource<I> for Option<T> {
	fn interpretation(&self, interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		match self {
			Some(t) => t.interpretation(interpretation),
			None => ResourceInterpretation::Uninterpreted(None),
		}
	}
}
