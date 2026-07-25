use rdfx::Interpretation;

use crate::{
	LinkedData, LinkedDataGraph, LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
/// Wrapper serializing its value as a reference: the subject is emitted
/// as an object, without its own properties.
pub struct Ref<T>(pub T);

impl<T> From<T> for Ref<T> {
	fn from(value: T) -> Self {
		Self(value)
	}
}

impl<I: Interpretation, T: LinkedDataResource<I>> LinkedDataResource<I> for Ref<T> {
	fn interpretation(&self, interpretation: &mut I) -> crate::ResourceInterpretation<'_, I> {
		T::reference_interpretation(&self.0, interpretation)
	}
}

impl<I: Interpretation, T> LinkedDataSubject<I> for Ref<T> {
	fn visit_subject<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: crate::SubjectVisitor<I>,
	{
		serializer.end()
	}
}

impl<I: Interpretation, T: LinkedDataResource<I>> LinkedDataPredicateObjects<I> for Ref<T> {
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: crate::PredicateObjectsVisitor<I>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}

impl<I: Interpretation, T: LinkedDataResource<I>> LinkedDataGraph<I> for Ref<T> {
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: crate::GraphVisitor<I>,
	{
		visitor.subject(self)?;
		visitor.end()
	}
}

impl<I: Interpretation, T: LinkedDataResource<I>> LinkedData<I> for Ref<T> {
	fn visit<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: crate::Visitor<I>,
	{
		visitor.default_graph(self)?;
		visitor.end()
	}
}
