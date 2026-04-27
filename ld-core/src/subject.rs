use iri_rs::IriBuf;
use rdf_rs::{
	dataset::PatternMatchingDataset,
	interpretation::{ReverseInterpretation, ReverseLocalInterpretation},
	BlankId, BlankIdBuf, Id, Interpretation,
};

use crate::{
	Context, FromLinkedDataError, LinkedDataGraph, LinkedDataPredicateObjects, LinkedDataResource,
};

/// Serialize a Linked-Data node.
pub trait LinkedDataSubject<I: Interpretation = ()> {
	fn visit_subject<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>;
}

impl<I: Interpretation> LinkedDataSubject<I> for () {
	fn visit_subject<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		serializer.end()
	}
}

impl<'a, I: Interpretation, T: ?Sized + LinkedDataSubject<I>> LinkedDataSubject<I> for &'a T {
	fn visit_subject<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		T::visit_subject(self, serializer)
	}
}

impl<I: Interpretation, T: ?Sized + LinkedDataSubject<I>> LinkedDataSubject<I> for Box<T> {
	fn visit_subject<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		T::visit_subject(self, serializer)
	}
}

impl<I: Interpretation> LinkedDataSubject<I> for IriBuf {
	fn visit_subject<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		serializer.end()
	}
}

impl<'i, I: Interpretation> LinkedDataSubject<I> for iri_rs::Iri<&'i str> {
	fn visit_subject<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		serializer.end()
	}
}

impl<I: Interpretation> LinkedDataSubject<I> for BlankId {
	fn visit_subject<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		serializer.end()
	}
}

impl<I: Interpretation> LinkedDataSubject<I> for BlankIdBuf {
	fn visit_subject<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		serializer.end()
	}
}

impl<I: Interpretation> LinkedDataSubject<I> for Id {
	fn visit_subject<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		serializer.end()
	}
}

pub trait SubjectVisitor<I: Interpretation> {
	type Ok;
	type Error;

	/// Visit a predicate of the graph.
	fn predicate<L, T>(&mut self, predicate: &L, objects: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LinkedDataResource<I>,
		T: ?Sized + LinkedDataPredicateObjects<I>;

	/// Visit a reverse predicate of the graph.
	fn reverse_predicate<L, T>(&mut self, predicate: &L, subjects: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LinkedDataResource<I>,
		T: ?Sized + LinkedDataPredicateObjects<I>;

	fn graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataGraph<I>;

	fn include<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + LinkedDataSubject<I>;

	fn end(self) -> Result<Self::Ok, Self::Error>;
}

impl<'s, I: Interpretation, S: SubjectVisitor<I>> SubjectVisitor<I> for &'s mut S {
	type Ok = ();
	type Error = S::Error;

	fn predicate<L, T>(&mut self, predicate: &L, objects: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LinkedDataResource<I>,
		T: ?Sized + LinkedDataPredicateObjects<I>,
	{
		S::predicate(self, predicate, objects)
	}

	fn reverse_predicate<L, T>(&mut self, predicate: &L, subjects: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LinkedDataResource<I>,
		T: ?Sized + LinkedDataPredicateObjects<I>,
	{
		S::reverse_predicate(self, predicate, subjects)
	}

	fn graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataGraph<I>,
	{
		S::graph(self, value)
	}

	fn include<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + LinkedDataSubject<I>,
	{
		S::include(self, value)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(())
	}
}

pub trait LinkedDataDeserializeSubject<I: Interpretation>: Sized
where
	I::Resource: rdf_rs::Resource,
{
	fn deserialize_subject_in<D>(
		interpretation: &I,
		dataset: &D,
		graph: Option<&I::Resource>,
		resource: &I::Resource,
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>
	where
		D: PatternMatchingDataset<Subject = I::Resource>;

	fn deserialize_subject<D>(
		interpretation: &I,
		dataset: &D,
		graph: Option<&I::Resource>,
		resource: &I::Resource,
	) -> Result<Self, FromLinkedDataError>
	where
		D: PatternMatchingDataset<Subject = I::Resource>,
	{
		Self::deserialize_subject_in(
			interpretation,
			dataset,
			graph,
			resource,
			Context::default(),
		)
	}
}

impl<I: Interpretation> LinkedDataDeserializeSubject<I> for IriBuf
where
	I: ReverseInterpretation,
	I::Resource: rdf_rs::Resource,
{
	fn deserialize_subject_in<D>(
		interpretation: &I,
		_dataset: &D,
		_graph: Option<&I::Resource>,
		resource: &I::Resource,
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>
	where
		D: PatternMatchingDataset<Subject = I::Resource>,
	{
		match interpretation.iris_of(resource).next() {
			Some(i) => Ok(i.into_owned()),
			None => Err(FromLinkedDataError::InvalidSubject {
				context: context.into_iris(interpretation),
				subject: None,
			}),
		}
	}
}

impl<I: Interpretation> LinkedDataDeserializeSubject<I> for BlankIdBuf
where
	I: ReverseInterpretation + ReverseLocalInterpretation,
	I::Resource: rdf_rs::Resource,
{
	fn deserialize_subject_in<D>(
		interpretation: &I,
		_dataset: &D,
		_graph: Option<&I::Resource>,
		resource: &I::Resource,
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>
	where
		D: PatternMatchingDataset<Subject = I::Resource>,
	{
		match interpretation.blank_ids_of(resource).next() {
			Some(b) => Ok(b.into_owned()),
			None => Err(FromLinkedDataError::InvalidSubject {
				context: context.into_iris(interpretation),
				subject: interpretation.iris_of(resource).next().map(|i| i.into_owned()),
			}),
		}
	}
}

impl<I: Interpretation> LinkedDataDeserializeSubject<I> for Id
where
	I: ReverseInterpretation + ReverseLocalInterpretation,
	I::Resource: rdf_rs::Resource,
{
	fn deserialize_subject_in<D>(
		interpretation: &I,
		_dataset: &D,
		_graph: Option<&I::Resource>,
		resource: &I::Resource,
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>
	where
		D: PatternMatchingDataset<Subject = I::Resource>,
	{
		if let Some(i) = interpretation.iris_of(resource).next() {
			return Ok(Id::Iri(i.into_owned()));
		}
		if let Some(b) = interpretation.blank_ids_of(resource).next() {
			return Ok(Id::BlankId(b.into_owned()));
		}
		Err(FromLinkedDataError::InvalidSubject {
			context: context.into_iris(interpretation),
			subject: None,
		})
	}
}

impl<I: Interpretation, T: LinkedDataDeserializeSubject<I>> LinkedDataDeserializeSubject<I>
	for Box<T>
where
	I::Resource: rdf_rs::Resource,
{
	fn deserialize_subject_in<D>(
		interpretation: &I,
		dataset: &D,
		graph: Option<&I::Resource>,
		resource: &I::Resource,
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>
	where
		D: PatternMatchingDataset<Subject = I::Resource>,
	{
		T::deserialize_subject_in(interpretation, dataset, graph, resource, context).map(Box::new)
	}
}
