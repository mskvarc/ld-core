use iri_rs::Iri;
use rdfx::Interpretation;

use crate::{
	BorrowedRdfTerm, CowRdfTerm, GraphVisitor, LinkedData, LinkedDataGraph,
	LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject, PredicateObjectsVisitor,
	ResourceInterpretation, SubjectVisitor, Visitor,
};

/// Binding of a value to a predicate, serialized on an anonymous subject.
pub struct AnonymousBinding<'a, T>(pub Iri<&'a str>, pub &'a T);

impl<'a, T> AnonymousBinding<'a, T> {
	/// Binds `value` to the `iri` predicate.
	pub fn new(iri: Iri<&'a str>, value: &'a T) -> Self {
		Self(iri, value)
	}
}

impl<'a, I: Interpretation, T> LinkedDataResource<I> for AnonymousBinding<'a, T> {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		ResourceInterpretation::Uninterpreted(None)
	}
}

impl<'a, I: Interpretation, T: LinkedDataPredicateObjects<I>> LinkedDataSubject<I>
	for AnonymousBinding<'a, T>
{
	fn visit_subject<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		serializer.predicate(&IriValue(self.0), self.1)?;
		serializer.end()
	}
}

impl<'a, I: Interpretation, T: LinkedDataPredicateObjects<I>> LinkedDataPredicateObjects<I>
	for AnonymousBinding<'a, T>
{
	fn visit_objects<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		serializer.object(self)?;
		serializer.end()
	}
}

impl<'a, I: Interpretation, T: LinkedDataPredicateObjects<I>> LinkedDataGraph<I>
	for AnonymousBinding<'a, T>
{
	fn visit_graph<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>,
	{
		serializer.subject(self)?;
		serializer.end()
	}
}

impl<'a, I: Interpretation, T: LinkedDataPredicateObjects<I>> LinkedData<I>
	for AnonymousBinding<'a, T>
{
	fn visit<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Visitor<I>,
	{
		serializer.default_graph(self)?;
		serializer.end()
	}
}

/// Wrapper exposing an `Iri<&str>` as a `LinkedDataResource` predicate.
struct IriValue<'a>(Iri<&'a str>);

impl<'a, I: Interpretation> LinkedDataResource<I> for IriValue<'a> {
	fn interpretation(&self, _interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(BorrowedRdfTerm::Iri(
			self.0,
		))))
	}
}
