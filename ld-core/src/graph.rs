use iri_rs::IriBuf;
use rdf_rs::Interpretation;

use crate::{LinkedData, LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject};

/// Serialize a Linked-Data graph.
pub trait LinkedDataGraph<I: Interpretation = ()> {
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

impl<'a, I: Interpretation, T: ?Sized + LinkedDataGraph<I>> LinkedDataGraph<I> for &'a T {
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

pub trait GraphVisitor<I: Interpretation> {
	type Ok;
	type Error;

	fn subject<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + LinkedDataSubject<I>;

	fn end(self) -> Result<Self::Ok, Self::Error>;
}

impl<'a, I: Interpretation, S: GraphVisitor<I>> GraphVisitor<I> for &'a mut S {
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

pub struct AnonymousGraph<T>(pub T);

impl<I: Interpretation, T> LinkedDataResource<I> for AnonymousGraph<T> {
	fn interpretation(
		&self,
		_interpretation: &mut I,
	) -> crate::ResourceInterpretation<'_, I> {
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

impl<I: Interpretation, T: LinkedDataGraph<I>> LinkedDataPredicateObjects<I>
	for AnonymousGraph<T>
{
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
