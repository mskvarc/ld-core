use iri_rs::IriBuf;
use rdfx::Interpretation;

use crate::{LinkedData, LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject};

/// Serialize a Linked-Data graph.
pub trait LinkedDataGraph<I: Interpretation = ()> {
	/// Visits the subjects of the graph represented by this value.
	fn visit_graph<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>;
}

impl<I: Interpretation> LinkedDataGraph<I> for () {
	fn visit_graph<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>,
	{
		visitor.end()
	}
}

impl<I: Interpretation, T: ?Sized + LinkedDataGraph<I>> LinkedDataGraph<I> for &T {
	fn visit_graph<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>,
	{
		T::visit_graph(self, visitor)
	}
}

impl<I: Interpretation, T: ?Sized + LinkedDataGraph<I>> LinkedDataGraph<I> for Box<T> {
	fn visit_graph<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>,
	{
		T::visit_graph(self, visitor)
	}
}

impl<I: Interpretation> LinkedDataGraph<I> for IriBuf {
	fn visit_graph<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>,
	{
		visitor.end()
	}
}

impl<I: Interpretation, T: LinkedDataSubject<I> + LinkedDataResource<I>> LinkedDataGraph<I>
	for [T]
{
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>,
	{
		for t in self {
			visitor.subject(t)?;
		}
		visitor.end()
	}
}

impl<I: Interpretation, T: LinkedDataSubject<I> + LinkedDataResource<I>> LinkedDataGraph<I>
	for Vec<T>
{
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>,
	{
		for t in self {
			visitor.subject(t)?;
		}
		visitor.end()
	}
}

/// Visitor over the subjects of an RDF graph.
pub trait GraphVisitor<I: Interpretation> {
	/// Value returned once the graph has been entirely visited.
	type Ok;
	/// Error type.
	type Error;

	/// Visits a subject of the graph.
	fn subject<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + LinkedDataSubject<I>;

	/// Ends the graph visit.
	fn end(self) -> Result<Self::Ok, Self::Error>;
}

impl<I: Interpretation, S: GraphVisitor<I>> GraphVisitor<I> for &mut S {
	type Ok = ();
	type Error = S::Error;

	fn subject<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + LinkedDataSubject<I>,
	{
		S::subject(self, value)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(())
	}
}

/// Wrapper serializing its value as an anonymous graph.
pub struct AnonymousGraph<T>(pub T);

impl<I: Interpretation, T> LinkedDataResource<I> for AnonymousGraph<T> {
	fn interpretation(&self, _interpretation: &mut I) -> crate::ResourceInterpretation<'_, I> {
		crate::ResourceInterpretation::Uninterpreted(None)
	}
}

impl<I: Interpretation, T: LinkedDataGraph<I>> LinkedDataSubject<I> for AnonymousGraph<T> {
	fn visit_subject<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error>
	where
		S: crate::SubjectVisitor<I>,
	{
		serializer.graph(&self.0)?;
		serializer.end()
	}
}

impl<I: Interpretation, T: LinkedDataGraph<I>> LinkedDataPredicateObjects<I> for AnonymousGraph<T> {
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: crate::PredicateObjectsVisitor<I>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}

impl<I: Interpretation, T: LinkedDataGraph<I>> LinkedDataGraph<I> for AnonymousGraph<T> {
	fn visit_graph<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>,
	{
		T::visit_graph(&self.0, visitor)
	}
}

impl<I: Interpretation, T: LinkedDataGraph<I>> LinkedData<I> for AnonymousGraph<T> {
	fn visit<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: crate::Visitor<I>,
	{
		visitor.named_graph(self)?;
		visitor.end()
	}
}
