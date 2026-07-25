use educe::Educe;
use iri_rs::IriBuf;
use rdfx::generator::LocalGeneratorInterpretation;
use rdfx::{
	Id, Interpretation, InterpretationMut, IsGraph, IsObject, IsPredicate, IsSubject,
	LocalGenerator, LocalTerm, Quad, Resource, Term,
	interpretation::{
		GenerativeInterpretation, LocalInterpretationMut, ReverseInterpretation,
		ReverseLocalInterpretation,
	},
};

use crate::{
	CowRdfTerm, GraphVisitor, InterpretedQuad, LinkedData, LinkedDataGraph, LinkedDataResource,
	LinkedDataSubject, OwnedRdfTerm, PredicateObjectsVisitor, RdfQuad, ResourceInterpretation,
	SubjectVisitor, Visitor,
};

/// Serializes `value` into quads whose components are resources of
/// `interpretation`.
pub fn to_interpreted_quads<I>(
	interpretation: &mut I,
	value: &impl LinkedData<I>,
) -> Result<Vec<InterpretedQuad<I>>, IntoQuadsError>
where
	I: Interpretation + InterpretationMut + LocalInterpretationMut + GenerativeInterpretation,
	I::Resource: Clone + Resource,
{
	value.visit(QuadSerializer {
		interpretation,
		domain: &mut InterpretationDomain,
		result: Vec::new(),
	})
}

/// Serializes `value` as a single subject in `graph`, returning its
/// resource and the quads describing it.
pub fn to_interpreted_subject_quads<I>(
	interpretation: &mut I,
	graph: Option<&I::Resource>,
	value: &(impl LinkedDataSubject<I> + LinkedDataResource<I>),
) -> Result<(I::Resource, Vec<InterpretedQuad<I>>), IntoQuadsError>
where
	I: Interpretation + InterpretationMut + LocalInterpretationMut + GenerativeInterpretation,
	I::Resource: Clone + Resource,
{
	let mut result = Vec::new();

	let i = value.interpretation(interpretation);
	let subject = InterpretationDomain.subject(interpretation, i)?;

	value.visit_subject(QuadPropertiesSerializer {
		interpretation,
		domain: &mut InterpretationDomain,
		graph,
		subject: SubjectOrObject::Subject(&subject),
		result: &mut result,
	})?;

	Ok((subject, result))
}

/// Serializes `value` as the content of `graph`, in the interpretation
/// domain.
pub fn to_interpreted_graph_quads<I>(
	interpretation: &mut I,
	value: &(impl LinkedDataGraph<I> + LinkedDataResource<I>),
) -> Result<(I::Resource, Vec<InterpretedQuad<I>>), IntoQuadsError>
where
	I: Interpretation + InterpretationMut + LocalInterpretationMut + GenerativeInterpretation,
	I::Resource: Clone + Resource,
{
	let mut result = Vec::new();

	let i = value.interpretation(interpretation);
	let graph = InterpretationDomain.graph(interpretation, i)?;

	value.visit_graph(QuadGraphSerializer {
		interpretation,
		domain: &mut InterpretationDomain,
		graph: Some(&graph),
		result: &mut result,
	})?;

	Ok((graph, result))
}

/// Serializes `value` into lexical quads, using the given interpretation to
/// resolve resources into IRIs, blank ids and literals.
pub fn to_lexical_quads_with<I>(
	interpretation: &mut I,
	value: &impl LinkedData<I>,
) -> Result<Vec<RdfQuad>, IntoQuadsError>
where
	I: Interpretation
		+ InterpretationMut
		+ ReverseInterpretation
		+ ReverseLocalInterpretation
		+ GenerativeInterpretation,
{
	let mut domain = LexicalDomain;

	value.visit(QuadSerializer {
		interpretation,
		domain: &mut domain,
		result: Vec::new(),
	})
}

/// Serializes `value` as a single subject in `graph`, returning its
/// identifier and the lexical quads describing it.
pub fn to_lexical_subject_quads_with<I>(
	interpretation: &mut I,
	graph: Option<&Id>,
	value: &(impl LinkedDataSubject<I> + LinkedDataResource<I>),
) -> Result<(Id, Vec<RdfQuad>), IntoQuadsError>
where
	I: Interpretation
		+ InterpretationMut
		+ ReverseInterpretation
		+ ReverseLocalInterpretation
		+ GenerativeInterpretation,
	I::Resource: Clone,
{
	let mut result = Vec::new();

	let i = value.interpretation(interpretation);
	let subject = LexicalDomain.subject(interpretation, i)?;

	value.visit_subject(QuadPropertiesSerializer {
		interpretation,
		domain: &mut LexicalDomain,
		graph,
		subject: SubjectOrObject::Subject(&subject),
		result: &mut result,
	})?;

	Ok((subject, result))
}

/// Serializes `value` into lexical quads, drawing blank node identifiers
/// from `generator`.
pub fn to_lexical_quads<G: LocalGenerator>(
	generator: G,
	value: &impl LinkedData<LocalGeneratorInterpretation<G>>,
) -> Result<Vec<RdfQuad>, IntoQuadsError> {
	let mut interpretation = LocalGeneratorInterpretation::new(generator);
	to_lexical_quads_with(&mut interpretation, value)
}

/// Serializes `value` as a single subject in `graph`, drawing blank node
/// identifiers from `generator`.
pub fn to_lexical_subject_quads<G: LocalGenerator>(
	generator: G,
	graph: Option<&Id>,
	value: &(
	     impl LinkedDataSubject<LocalGeneratorInterpretation<G>>
	     + LinkedDataResource<LocalGeneratorInterpretation<G>>
	 ),
) -> Result<(Id, Vec<RdfQuad>), IntoQuadsError> {
	let mut interpretation = LocalGeneratorInterpretation::new(generator);
	to_lexical_subject_quads_with(&mut interpretation, graph, value)
}

/// Serializes `value` into quads, using the given interpretation.
///
/// Alias of [`to_lexical_quads_with`].
pub fn to_quads_with<I>(
	interpretation: &mut I,
	value: &impl LinkedData<I>,
) -> Result<Vec<RdfQuad>, IntoQuadsError>
where
	I: Interpretation
		+ InterpretationMut
		+ ReverseInterpretation
		+ ReverseLocalInterpretation
		+ GenerativeInterpretation,
{
	to_lexical_quads_with(interpretation, value)
}

/// Serializes `value` into quads, drawing blank node identifiers from
/// `generator`.
///
/// Alias of [`to_lexical_quads`].
pub fn to_quads<G: LocalGenerator>(
	generator: G,
	value: &impl LinkedData<LocalGeneratorInterpretation<G>>,
) -> Result<Vec<RdfQuad>, IntoQuadsError> {
	to_lexical_quads(generator, value)
}

#[derive(Debug, thiserror::Error)]
/// Error raised while serializing a value into quads.
pub enum IntoQuadsError {
	#[error("invalid graph label")]
	/// A resource used as a graph label is neither an IRI nor a blank node.
	Graph,

	#[error("invalid subject")]
	/// A resource used as a subject is neither an IRI nor a blank node.
	Subject,

	#[error("invalid predicate")]
	/// A resource used as a predicate is not an IRI.
	Predicate,

	#[error("missing lexical representation")]
	/// A resource has no lexical representation in the interpretation.
	MissingLexicalRepresentation,
}

trait Domain<I: Interpretation> {
	type Subject: Clone + IsSubject + IsGraph;
	type Predicate: Clone + IsPredicate;
	type Object: IsObject;
	type ObjectRef<'a>: Copy
	where
		I::Resource: 'a,
		Self: 'a;

	fn subject(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Subject, IntoQuadsError>;

	fn predicate(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Predicate, IntoQuadsError>;

	fn object(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Object, IntoQuadsError>;

	fn graph(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Subject, IntoQuadsError>;

	fn object_as_subject(&self, object: &Self::Object) -> Result<Self::Subject, IntoQuadsError>;

	fn subject_as_object<'a>(
		&self,
		subject: &'a Self::Subject,
	) -> Result<Self::ObjectRef<'a>, IntoQuadsError>
	where
		I::Resource: 'a;

	fn object_as_ref<'a>(object: &'a Self::Object) -> Self::ObjectRef<'a>
	where
		I::Resource: 'a;

	fn cloned_object_ref<'a>(object_ref: Self::ObjectRef<'a>) -> Self::Object
	where
		I::Resource: 'a;
}

type DomainQuad<I, D> = Quad<
	<D as Domain<I>>::Subject,
	<D as Domain<I>>::Predicate,
	<D as Domain<I>>::Object,
	<D as Domain<I>>::Subject,
>;

struct InterpretationDomain;

#[allow(clippy::type_complexity)]
fn resource_to_local_term<I>(
	interpretation: &I,
	r: &I::Resource,
) -> Result<LocalTerm, IntoQuadsError>
where
	I: ReverseInterpretation + ReverseLocalInterpretation,
{
	if let Some(iri) = interpretation.iris_of(r).next() {
		return Ok(LocalTerm::iri(iri.into_owned()));
	}

	if let Some(lit) = interpretation.literals_of(r).next() {
		return Ok(LocalTerm::literal(lit.into_owned()));
	}

	if let Some(blank_id) = interpretation.blank_ids_of(r).next() {
		return Ok(LocalTerm::BlankId(blank_id.into_owned()));
	}

	Err(IntoQuadsError::MissingLexicalRepresentation)
}

fn resource_to_id<I>(interpretation: &I, r: &I::Resource) -> Result<Id, IntoQuadsError>
where
	I: ReverseInterpretation + ReverseLocalInterpretation,
{
	if let Some(iri) = interpretation.iris_of(r).next() {
		return Ok(Id::Iri(iri.into_owned()));
	}

	if let Some(blank_id) = interpretation.blank_ids_of(r).next() {
		return Ok(Id::BlankId(blank_id.into_owned()));
	}

	Err(IntoQuadsError::MissingLexicalRepresentation)
}

fn cow_term_into_id(t: CowRdfTerm<'_>) -> Option<Id> {
	let owned = t.into_owned();
	owned.into_id()
}

fn cow_term_into_iri(t: CowRdfTerm<'_>) -> Option<IriBuf> {
	match t.into_owned() {
		OwnedRdfTerm::Iri(i) => Some(i),
		_ => None,
	}
}

impl<I> Domain<I> for InterpretationDomain
where
	I: Interpretation + InterpretationMut + LocalInterpretationMut + GenerativeInterpretation,
	I::Resource: Clone + Resource,
{
	type Subject = I::Resource;
	type Predicate = I::Resource;
	type Object = I::Resource;
	type ObjectRef<'a>
		= &'a I::Resource
	where
		I::Resource: 'a;

	fn subject(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Subject, IntoQuadsError> {
		match value {
			ResourceInterpretation::Interpreted(r) => Ok(r.clone()),
			ResourceInterpretation::Uninterpreted(u) => match u {
				Some(t) => match t.into_owned() {
					OwnedRdfTerm::Iri(i) => {
						Ok(interpretation.insert_iri(std::borrow::Cow::Owned(i)))
					}
					OwnedRdfTerm::BlankId(b) => {
						Ok(interpretation.insert_blank_id(std::borrow::Cow::Owned(b)))
					}
					OwnedRdfTerm::Literal(_) => Err(IntoQuadsError::Subject),
				},
				None => Ok(interpretation.new_resource()),
			},
		}
	}

	fn predicate(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Predicate, IntoQuadsError> {
		match value {
			ResourceInterpretation::Interpreted(r) => Ok(r.clone()),
			ResourceInterpretation::Uninterpreted(u) => match u {
				Some(t) => match t.into_owned() {
					OwnedRdfTerm::Iri(i) => {
						Ok(interpretation.insert_iri(std::borrow::Cow::Owned(i)))
					}
					_ => Err(IntoQuadsError::Predicate),
				},
				None => Err(IntoQuadsError::Predicate),
			},
		}
	}

	fn object(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Object, IntoQuadsError> {
		match value {
			ResourceInterpretation::Interpreted(r) => Ok(r.clone()),
			ResourceInterpretation::Uninterpreted(u) => match u {
				Some(t) => match t.into_owned() {
					OwnedRdfTerm::Iri(i) => {
						Ok(interpretation.insert_iri(std::borrow::Cow::Owned(i)))
					}
					OwnedRdfTerm::BlankId(b) => {
						Ok(interpretation.insert_blank_id(std::borrow::Cow::Owned(b)))
					}
					OwnedRdfTerm::Literal(l) => Ok(l.embed_into(interpretation)),
				},
				None => Ok(interpretation.new_resource()),
			},
		}
	}

	fn graph(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Subject, IntoQuadsError> {
		match value {
			ResourceInterpretation::Interpreted(r) => Ok(r.clone()),
			ResourceInterpretation::Uninterpreted(u) => match u {
				Some(t) => match t.into_owned() {
					OwnedRdfTerm::Iri(i) => {
						Ok(interpretation.insert_iri(std::borrow::Cow::Owned(i)))
					}
					OwnedRdfTerm::BlankId(b) => {
						Ok(interpretation.insert_blank_id(std::borrow::Cow::Owned(b)))
					}
					OwnedRdfTerm::Literal(_) => Err(IntoQuadsError::Graph),
				},
				None => Ok(interpretation.new_resource()),
			},
		}
	}

	fn object_as_subject(&self, object: &Self::Object) -> Result<Self::Subject, IntoQuadsError> {
		Ok(object.clone())
	}

	fn subject_as_object<'a>(
		&self,
		subject: &'a Self::Subject,
	) -> Result<Self::ObjectRef<'a>, IntoQuadsError>
	where
		I::Resource: 'a,
	{
		Ok(subject)
	}

	fn object_as_ref<'a>(object: &'a Self::Object) -> Self::ObjectRef<'a>
	where
		I::Resource: 'a,
	{
		object
	}

	fn cloned_object_ref<'a>(object_ref: Self::ObjectRef<'a>) -> Self::Object
	where
		I::Resource: 'a,
	{
		object_ref.clone()
	}
}

struct LexicalDomain;

fn lexical_term_into_local(t: CowRdfTerm<'_>) -> LocalTerm {
	match t.into_owned() {
		OwnedRdfTerm::Iri(i) => LocalTerm::iri(i),
		OwnedRdfTerm::BlankId(b) => LocalTerm::BlankId(b),
		OwnedRdfTerm::Literal(l) => LocalTerm::literal(l.into_lexical()),
	}
}

impl<I> Domain<I> for LexicalDomain
where
	I: Interpretation
		+ InterpretationMut
		+ ReverseInterpretation
		+ ReverseLocalInterpretation
		+ GenerativeInterpretation,
{
	type Subject = Id;
	type Predicate = IriBuf;
	type Object = LocalTerm;
	type ObjectRef<'a>
		= LocalTermRefCopy<'a>
	where
		I::Resource: 'a;

	fn subject(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Subject, IntoQuadsError> {
		match value {
			ResourceInterpretation::Interpreted(r) => resource_to_id(interpretation, r),
			ResourceInterpretation::Uninterpreted(u) => match u {
				Some(t) => cow_term_into_id(t).ok_or(IntoQuadsError::Subject),
				None => {
					let r = interpretation.new_resource();
					resource_to_id(interpretation, &r)
				}
			},
		}
	}

	fn predicate(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Predicate, IntoQuadsError> {
		match value {
			ResourceInterpretation::Interpreted(r) => match interpretation.iris_of(r).next() {
				Some(iri) => Ok(iri.into_owned()),
				None => Err(IntoQuadsError::Predicate),
			},
			ResourceInterpretation::Uninterpreted(u) => match u {
				Some(t) => cow_term_into_iri(t).ok_or(IntoQuadsError::Predicate),
				None => Err(IntoQuadsError::Predicate),
			},
		}
	}

	fn object(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Object, IntoQuadsError> {
		match value {
			ResourceInterpretation::Interpreted(r) => resource_to_local_term(interpretation, r),
			ResourceInterpretation::Uninterpreted(u) => match u {
				Some(t) => Ok(lexical_term_into_local(t)),
				None => {
					let r = interpretation.new_resource();
					resource_to_local_term(interpretation, &r)
				}
			},
		}
	}

	fn graph(
		&mut self,
		interpretation: &mut I,
		value: ResourceInterpretation<I>,
	) -> Result<Self::Subject, IntoQuadsError> {
		match value {
			ResourceInterpretation::Interpreted(r) => resource_to_id(interpretation, r),
			ResourceInterpretation::Uninterpreted(u) => match u {
				Some(t) => cow_term_into_id(t).ok_or(IntoQuadsError::Graph),
				None => {
					let r = interpretation.new_resource();
					resource_to_id(interpretation, &r)
				}
			},
		}
	}

	fn object_as_subject(&self, object: &Self::Object) -> Result<Self::Subject, IntoQuadsError> {
		match object {
			LocalTerm::BlankId(b) => Ok(Id::BlankId(b.clone())),
			LocalTerm::Named(Term::Iri(i)) => Ok(Id::Iri(i.clone())),
			_ => Err(IntoQuadsError::Subject),
		}
	}

	fn subject_as_object<'a>(
		&self,
		subject: &'a Self::Subject,
	) -> Result<Self::ObjectRef<'a>, IntoQuadsError>
	where
		I::Resource: 'a,
	{
		Ok(LocalTermRefCopy::Id(subject))
	}

	fn object_as_ref<'a>(object: &'a Self::Object) -> Self::ObjectRef<'a>
	where
		I::Resource: 'a,
	{
		LocalTermRefCopy::Term(object)
	}

	fn cloned_object_ref<'a>(object_ref: Self::ObjectRef<'a>) -> Self::Object
	where
		I::Resource: 'a,
	{
		match object_ref {
			LocalTermRefCopy::Id(id) => match id {
				Id::Iri(i) => LocalTerm::iri(i.clone()),
				Id::BlankId(b) => LocalTerm::BlankId(b.clone()),
			},
			LocalTermRefCopy::Term(t) => t.clone(),
		}
	}
}

#[derive(Clone, Copy)]
enum LocalTermRefCopy<'a> {
	Id(&'a Id),
	Term(&'a LocalTerm),
}

/// A simple serializer generating a list of `Quad`s.
struct QuadSerializer<'a, I: Interpretation, D: Domain<I>> {
	interpretation: &'a mut I,
	domain: &'a mut D,
	result: Vec<DomainQuad<I, D>>,
}

impl<'a, I: Interpretation, D: Domain<I>> Visitor<I> for QuadSerializer<'a, I, D> {
	type Ok = Vec<DomainQuad<I, D>>;
	type Error = IntoQuadsError;

	fn default_graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + crate::LinkedDataGraph<I>,
	{
		let graph_serializer = QuadGraphSerializer {
			interpretation: self.interpretation,
			domain: self.domain,
			result: &mut self.result,
			graph: None,
		};

		value.visit_graph(graph_serializer)
	}

	fn named_graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + crate::LinkedDataGraph<I>,
	{
		let i = value.interpretation(self.interpretation);
		let graph = self.domain.graph(self.interpretation, i)?;

		let graph_serializer = QuadGraphSerializer {
			interpretation: self.interpretation,
			domain: self.domain,
			result: &mut self.result,
			graph: Some(&graph),
		};

		value.visit_graph(graph_serializer)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}

struct QuadGraphSerializer<'a, I: Interpretation, D: Domain<I>> {
	interpretation: &'a mut I,
	domain: &'a mut D,
	result: &'a mut Vec<DomainQuad<I, D>>,
	graph: Option<&'a D::Subject>,
}

impl<'a, I: Interpretation, D: Domain<I>> GraphVisitor<I> for QuadGraphSerializer<'a, I, D> {
	type Ok = ();
	type Error = IntoQuadsError;

	fn subject<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + crate::LinkedDataSubject<I>,
	{
		let i = value.interpretation(self.interpretation);
		let term = self.domain.subject(self.interpretation, i)?;

		let properties_serializer = QuadPropertiesSerializer {
			interpretation: self.interpretation,
			domain: self.domain,
			result: self.result,
			graph: self.graph,
			subject: SubjectOrObject::Subject(&term),
		};

		value.visit_subject(properties_serializer)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(())
	}
}

struct QuadPropertiesSerializer<'a, I: Interpretation, D: Domain<I>> {
	interpretation: &'a mut I,
	domain: &'a mut D,
	result: &'a mut Vec<DomainQuad<I, D>>,
	graph: Option<&'a D::Subject>,
	subject: SubjectOrObject<'a, I, D>,
}

#[derive(Educe)]
#[educe(Clone, Copy)]
enum SubjectOrObject<'a, I: Interpretation, D: Domain<I>> {
	Subject(&'a D::Subject),
	Object(&'a D::Object),
}

impl<'a, I: Interpretation, D: Domain<I>> SubjectOrObject<'a, I, D> {
	fn into_subject(self, domain: &D) -> Result<std::borrow::Cow<'a, D::Subject>, IntoQuadsError>
	where
		D::Subject: Clone,
	{
		match self {
			Self::Subject(s) => Ok(std::borrow::Cow::Borrowed(s)),
			Self::Object(o) => domain.object_as_subject(o).map(std::borrow::Cow::Owned),
		}
	}

	fn into_object(self, domain: &D) -> Result<D::ObjectRef<'a>, IntoQuadsError> {
		match self {
			Self::Subject(s) => domain.subject_as_object(s),
			Self::Object(o) => Ok(D::object_as_ref(o)),
		}
	}
}

impl<'a, I: Interpretation, D: Domain<I>> SubjectVisitor<I> for QuadPropertiesSerializer<'a, I, D> {
	type Ok = ();
	type Error = IntoQuadsError;

	fn predicate<L, T>(&mut self, predicate: &L, value: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LinkedDataResource<I>,
		T: ?Sized + crate::LinkedDataPredicateObjects<I>,
	{
		let subject = self.subject.into_subject(self.domain)?;

		let i = predicate.interpretation(self.interpretation);
		let term = self.domain.predicate(self.interpretation, i)?;

		let objects_serializer = ObjectsSerializer {
			interpretation: self.interpretation,
			domain: self.domain,
			result: self.result,
			graph: self.graph,
			subject: subject.as_ref(),
			predicate: term,
		};

		value.visit_objects(objects_serializer)
	}

	fn reverse_predicate<L, T>(&mut self, predicate: &L, subjects: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LinkedDataResource<I>,
		T: ?Sized + crate::LinkedDataPredicateObjects<I>,
	{
		let object = self.subject.into_object(self.domain)?;

		let i = predicate.interpretation(self.interpretation);
		let term = self.domain.predicate(self.interpretation, i)?;

		let subjects_serializer = ReversePredicateSerializer {
			interpretation: self.interpretation,
			domain: self.domain,
			result: self.result,
			graph: self.graph,
			object,
			predicate: term,
		};

		subjects.visit_objects(subjects_serializer)
	}

	fn graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataGraph<I>,
	{
		let graph = self.subject.into_subject(self.domain)?;

		let graph_serializer = QuadGraphSerializer {
			interpretation: self.interpretation,
			domain: self.domain,
			result: self.result,
			graph: Some(graph.as_ref()),
		};

		value.visit_graph(graph_serializer)
	}

	fn include<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + LinkedDataSubject<I>,
	{
		let i = value.interpretation(self.interpretation);
		let subject = self.domain.subject(self.interpretation, i)?;

		value.visit_subject(QuadPropertiesSerializer {
			interpretation: self.interpretation,
			domain: self.domain,
			graph: self.graph,
			subject: SubjectOrObject::Subject(&subject),
			result: self.result,
		})?;

		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(())
	}
}

struct ObjectsSerializer<'a, I: Interpretation, D: Domain<I>> {
	interpretation: &'a mut I,
	domain: &'a mut D,
	result: &'a mut Vec<DomainQuad<I, D>>,
	graph: Option<&'a D::Subject>,
	subject: &'a D::Subject,
	predicate: D::Predicate,
}

impl<'a, I: Interpretation, D: Domain<I>> PredicateObjectsVisitor<I>
	for ObjectsSerializer<'a, I, D>
{
	type Ok = ();
	type Error = IntoQuadsError;

	fn object<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + crate::LinkedDataSubject<I>,
	{
		let i = value.interpretation(self.interpretation);
		let term = self.domain.object(self.interpretation, i)?;
		let subject_serializer = QuadPropertiesSerializer {
			interpretation: self.interpretation,
			domain: self.domain,
			result: self.result,
			graph: self.graph,
			subject: SubjectOrObject::Object(&term),
		};

		value.visit_subject(subject_serializer)?;
		self.result.push(Quad(
			self.subject.clone(),
			self.predicate.clone(),
			term,
			self.graph.cloned(),
		));
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(())
	}
}

struct ReversePredicateSerializer<'a, I: Interpretation, D: Domain<I>> {
	interpretation: &'a mut I,
	domain: &'a mut D,
	result: &'a mut Vec<DomainQuad<I, D>>,
	graph: Option<&'a D::Subject>,
	object: D::ObjectRef<'a>,
	predicate: D::Predicate,
}

impl<'a, I: Interpretation, D: Domain<I>> PredicateObjectsVisitor<I>
	for ReversePredicateSerializer<'a, I, D>
{
	type Ok = ();
	type Error = IntoQuadsError;

	fn object<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I> + crate::LinkedDataSubject<I>,
	{
		let i = value.interpretation(self.interpretation);
		let subject = self.domain.subject(self.interpretation, i)?;

		let subject_serializer = QuadPropertiesSerializer {
			interpretation: self.interpretation,
			domain: self.domain,
			result: self.result,
			graph: self.graph,
			subject: SubjectOrObject::Subject(&subject),
		};

		value.visit_subject(subject_serializer)?;
		self.result.push(Quad(
			subject,
			self.predicate.clone(),
			D::cloned_object_ref(self.object),
			self.graph.cloned(),
		));
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(())
	}
}
