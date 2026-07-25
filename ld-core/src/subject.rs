use iri_rs::IriBuf;
use rdfx::{
	BlankId, BlankIdBuf, Id, Interpretation,
	dataset::PatternMatchingDataset,
	interpretation::{ReverseInterpretation, ReverseLocalInterpretation},
};

use crate::{
	Context, FromLinkedDataError, LinkedDataGraph, LinkedDataPredicateObjects, LinkedDataResource,
};

/// Serialize a Linked-Data node.
pub trait LinkedDataSubject<I: Interpretation = ()> {
	/// Visits the properties of the subject represented by this value.
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

impl<I: Interpretation, T: ?Sized + LinkedDataSubject<I>> LinkedDataSubject<I> for &T {
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

impl<I: Interpretation> LinkedDataSubject<I> for iri_rs::Iri<&str> {
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

/// Visitor over the properties of an RDF subject.
pub trait SubjectVisitor<I: Interpretation> {
	/// Value returned once the subject has been entirely visited.
	type Ok;
	/// Error type.
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

	/// Visits a graph named by the subject.
	fn graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataGraph<I>;

	/// Includes another subject in the same graph.
	fn include<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + LinkedDataSubject<I>;

	/// Ends the subject visit.
	fn end(self) -> Result<Self::Ok, Self::Error>;
}

impl<I: Interpretation, S: SubjectVisitor<I>> SubjectVisitor<I> for &mut S {
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

/// Type that can be deserialized from a subject of an RDF dataset.
pub trait LinkedDataDeserializeSubject<I: Interpretation>: Sized
where
	I::Resource: rdfx::Resource,
{
	/// Deserializes a value from `resource`, reporting errors against
	/// `context`.
	fn deserialize_subject_in<D>(
		interpretation: &I,
		dataset: &D,
		graph: Option<&I::Resource>,
		resource: &I::Resource,
		context: Context<I>,
	) -> Result<Self, FromLinkedDataError>
	where
		D: PatternMatchingDataset<Subject = I::Resource>;

	/// Deserializes a value from `resource`.
	fn deserialize_subject<D>(
		interpretation: &I,
		dataset: &D,
		graph: Option<&I::Resource>,
		resource: &I::Resource,
	) -> Result<Self, FromLinkedDataError>
	where
		D: PatternMatchingDataset<Subject = I::Resource>,
	{
		Self::deserialize_subject_in(interpretation, dataset, graph, resource, Context::default())
	}
}

impl<I: Interpretation> LinkedDataDeserializeSubject<I> for IriBuf
where
	I: ReverseInterpretation,
	I::Resource: rdfx::Resource,
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
	I::Resource: rdfx::Resource,
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
				subject: interpretation
					.iris_of(resource)
					.next()
					.map(|i| i.into_owned()),
			}),
		}
	}
}

impl<I: Interpretation> LinkedDataDeserializeSubject<I> for Id
where
	I: ReverseInterpretation + ReverseLocalInterpretation,
	I::Resource: rdfx::Resource,
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
	I::Resource: rdfx::Resource,
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
