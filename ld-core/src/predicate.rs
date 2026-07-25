use iri_rs::IriBuf;
use rdfx::{
	BlankId, BlankIdBuf, Id, Interpretation,
	dataset::PatternMatchingDataset,
	interpretation::{ReverseInterpretation, ReverseLocalInterpretation},
};

use crate::{
	Context, FromLinkedDataError, LinkedDataDeserializeSubject, LinkedDataResource,
	LinkedDataSubject,
};

/// Type representing the objects of an RDF subject's predicate binding.
pub trait LinkedDataPredicateObjects<I: Interpretation = ()> {
	/// Visits the objects this predicate points at.
	fn visit_objects<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>;
}

impl<I: Interpretation> LinkedDataPredicateObjects<I> for () {
	fn visit_objects<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		visitor.end()
	}
}

impl<I: Interpretation, T: ?Sized + LinkedDataPredicateObjects<I>> LinkedDataPredicateObjects<I>
	for &T
{
	fn visit_objects<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		T::visit_objects(self, visitor)
	}
}

impl<I: Interpretation, T: ?Sized + LinkedDataPredicateObjects<I>> LinkedDataPredicateObjects<I>
	for Box<T>
{
	fn visit_objects<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		T::visit_objects(self, visitor)
	}
}

impl<I: Interpretation, T: LinkedDataSubject<I> + LinkedDataResource<I>>
	LinkedDataPredicateObjects<I> for Option<T>
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		if let Some(t) = self {
			visitor.object(t)?;
		}
		visitor.end()
	}
}

impl<I: Interpretation, T: LinkedDataSubject<I> + LinkedDataResource<I>>
	LinkedDataPredicateObjects<I> for [T]
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		for t in self {
			visitor.object(t)?;
		}
		visitor.end()
	}
}

impl<I: Interpretation, T: LinkedDataSubject<I> + LinkedDataResource<I>>
	LinkedDataPredicateObjects<I> for Vec<T>
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		for t in self {
			visitor.object(t)?;
		}
		visitor.end()
	}
}

impl<I: Interpretation> LinkedDataPredicateObjects<I> for IriBuf {
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}

impl<I: Interpretation> LinkedDataPredicateObjects<I> for iri_rs::Iri<&str> {
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}

impl<I: Interpretation> LinkedDataPredicateObjects<I> for BlankId {
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}

impl<I: Interpretation> LinkedDataPredicateObjects<I> for BlankIdBuf {
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}

impl<I: Interpretation> LinkedDataPredicateObjects<I> for Id {
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}

/// Visitor over the objects of a predicate.
pub trait PredicateObjectsVisitor<I: Interpretation> {
	/// Value returned once every object has been visited.
	type Ok;
	/// Error type.
	type Error;

	/// Visits an object of the predicate.
	fn object<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + LinkedDataSubject<I>;

	/// Ends the objects visit.
	fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Type that can be deserialized from the objects of a predicate.
pub trait LinkedDataDeserializePredicateObjects<I: Interpretation>: Sized
where
	I::Resource: rdfx::Resource,
{
	/// Deserializes a value from `objects`, reporting errors against
	/// `context`.
	fn deserialize_objects_in<'a, D>(
		interpretation: &I,
		dataset: &D,
		graph: Option<&I::Resource>,
		objects: impl IntoIterator<Item = &'a I::Resource>,
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>
	where
		I::Resource: 'a,
		D: PatternMatchingDataset<Subject = I::Resource>;

	/// Deserializes a value from `objects`.
	fn deserialize_objects<'a, D>(
		interpretation: &I,
		dataset: &D,
		graph: Option<&I::Resource>,
		objects: impl IntoIterator<Item = &'a I::Resource>,
	) -> Result<Self, FromLinkedDataError>
	where
		I::Resource: 'a,
		D: PatternMatchingDataset<Subject = I::Resource>,
	{
		Self::deserialize_objects_in(interpretation, dataset, graph, objects, Context::default())
	}
}

macro_rules! deserialize_single_object {
	() => {
		fn deserialize_objects_in<'a, D>(
			interpretation: &I,
			dataset: &D,
			graph: Option<&I::Resource>,
			objects: impl IntoIterator<Item = &'a I::Resource>,
			context: $crate::Context<I>,
		) -> Result<Self, FromLinkedDataError>
		where
			I::Resource: 'a,
			D: PatternMatchingDataset<Subject = I::Resource>,
		{
			use crate::LinkedDataDeserializeSubject;
			let mut objects = objects.into_iter();
			match objects.next() {
				Some(object) => {
					if objects.next().is_none() {
						Self::deserialize_subject_in(
							interpretation,
							dataset,
							graph,
							object,
							context,
						)
					} else {
						Err(FromLinkedDataError::TooManyValues(
							context.into_iris(interpretation),
						))
					}
				}
				None => Err(FromLinkedDataError::MissingRequiredValue(
					context.into_iris(interpretation),
				)),
			}
		}
	};
}

impl<I: Interpretation> LinkedDataDeserializePredicateObjects<I> for IriBuf
where
	I: ReverseInterpretation,
	I::Resource: rdfx::Resource,
{
	deserialize_single_object!();
}

impl<I: Interpretation> LinkedDataDeserializePredicateObjects<I> for BlankIdBuf
where
	I: ReverseInterpretation + ReverseLocalInterpretation,
	I::Resource: rdfx::Resource,
{
	deserialize_single_object!();
}

impl<I: Interpretation> LinkedDataDeserializePredicateObjects<I> for Id
where
	I: ReverseInterpretation + ReverseLocalInterpretation,
	I::Resource: rdfx::Resource,
{
	deserialize_single_object!();
}

impl<I: Interpretation, T: LinkedDataDeserializePredicateObjects<I>>
	LinkedDataDeserializePredicateObjects<I> for Box<T>
where
	I::Resource: rdfx::Resource,
{
	fn deserialize_objects_in<'a, D>(
		interpretation: &I,
		dataset: &D,
		graph: Option<&I::Resource>,
		objects: impl IntoIterator<Item = &'a I::Resource>,
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>
	where
		I::Resource: 'a,
		D: PatternMatchingDataset<Subject = I::Resource>,
	{
		T::deserialize_objects_in(interpretation, dataset, graph, objects, context).map(Box::new)
	}
}

impl<I: Interpretation, T: LinkedDataDeserializeSubject<I>> LinkedDataDeserializePredicateObjects<I>
	for Option<T>
where
	I: ReverseInterpretation,
	I::Resource: rdfx::Resource,
{
	fn deserialize_objects_in<'a, D>(
		interpretation: &I,
		dataset: &D,
		graph: Option<&I::Resource>,
		objects: impl IntoIterator<Item = &'a I::Resource>,
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>
	where
		I::Resource: 'a,
		D: PatternMatchingDataset<Subject = I::Resource>,
	{
		let mut objects = objects.into_iter();
		match objects.next() {
			Some(object) => {
				if objects.next().is_none() {
					T::deserialize_subject_in(interpretation, dataset, graph, object, context)
						.map(Some)
				} else {
					Err(FromLinkedDataError::TooManyValues(
						context.into_iris(interpretation),
					))
				}
			}
			None => Ok(None),
		}
	}
}
