use std::hash::Hash;

use crate::{
	GraphVisitor, LinkedDataGraph, LinkedDataPredicateObjects, LinkedDataResource,
	LinkedDataSubject, PredicateObjectsVisitor, ResourceInterpretation, SubjectVisitor,
};
use rdfx::{
	Interpretation, Quad,
	dataset::{
		DatasetView, Graph, PatternMatchingDataset, PredicateTraversableDataset,
		SubjectTraversableDataset,
	},
};

impl<'a, I: Interpretation, D> LinkedDataGraph<I> for DatasetView<'a, D>
where
	I::Resource: Eq + Hash + LinkedDataResource<I> + rdfx::Resource,
	D: rdfx::dataset::Dataset
		+ SubjectTraversableDataset
		+ PredicateTraversableDataset
		+ PatternMatchingDataset
		+ Graph<Subject = I::Resource>,
	<D as rdfx::dataset::Dataset>::Graph: PartialEq<I::Resource>,
{
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>,
	{
		let mut visited_subjects = im::HashSet::new();
		let mut visited_graphs = im::HashSet::new();
		if let Some(g) = self.graph {
			visited_graphs.insert(g);
		}

		let mut graph_subjects = Vec::new();

		for subject in self.dataset.subjects() {
			if self.graph.is_none()
				|| self
					.dataset
					.quad_pattern_matching(Quad(Some(subject), None, None, Some(self.graph)).into())
					.next()
					.is_some()
			{
				visited_subjects.insert(subject);
				graph_subjects.push(subject);
			}
		}

		for subject in graph_subjects {
			visitor.subject(&Subject::new(
				self.dataset,
				self.graph,
				subject,
				&visited_subjects,
				&visited_graphs,
				true,
			))?;
		}

		visitor.end()
	}
}

struct PredicateObjects<'d, 'v, D: rdfx::dataset::Dataset>
where
	D: PatternMatchingDataset,
	D::Subject: rdfx::Resource,
{
	dataset: &'d D,
	graph: Option<&'d D::Subject>,
	subject: &'d D::Subject,
	predicate: &'d D::Subject,
	visited_subjects: &'v im::HashSet<&'d D::Subject>,
	visited_graphs: &'v im::HashSet<&'d D::Subject>,
}

impl<'d, 'v, I: Interpretation, D> LinkedDataPredicateObjects<I> for PredicateObjects<'d, 'v, D>
where
	I::Resource: Eq + Hash + LinkedDataResource<I> + rdfx::Resource,
	D: rdfx::dataset::Dataset
		+ SubjectTraversableDataset
		+ PredicateTraversableDataset
		+ PatternMatchingDataset
		+ Graph<Subject = I::Resource>,
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: PredicateObjectsVisitor<I>,
	{
		for object in self
			.dataset
			.quad_objects(self.graph, self.subject, self.predicate)
		{
			visitor.object(&Object {
				dataset: self.dataset,
				graph: self.graph,
				object,
				visited_subjects: self.visited_subjects,
				visited_graphs: self.visited_graphs,
			})?;
		}

		visitor.end()
	}
}

impl<'a, 'v, D, I: Interpretation> LinkedDataResource<I> for Object<'a, 'v, D>
where
	I::Resource: LinkedDataResource<I>,
	D: rdfx::dataset::Dataset + Graph<Subject = I::Resource> + PatternMatchingDataset,
	D::Subject: rdfx::Resource,
{
	fn interpretation(&self, interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		self.object.interpretation(interpretation)
	}
}

struct Object<'a, 'v, D: rdfx::dataset::Dataset>
where
	D: PatternMatchingDataset,
	D::Subject: rdfx::Resource,
{
	dataset: &'a D,
	graph: Option<&'a D::Subject>,
	object: &'a D::Subject,
	visited_subjects: &'v im::HashSet<&'a D::Subject>,
	visited_graphs: &'v im::HashSet<&'a D::Subject>,
}

impl<'a, 'v, I: Interpretation, D> LinkedDataSubject<I> for Object<'a, 'v, D>
where
	I::Resource: Eq + Hash + LinkedDataResource<I> + rdfx::Resource,
	D: rdfx::dataset::Dataset
		+ SubjectTraversableDataset
		+ PredicateTraversableDataset
		+ PatternMatchingDataset
		+ Graph<Subject = I::Resource>,
{
	fn visit_subject<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		let subject = self.object;

		let mut visited_subjects = self.visited_subjects.clone();
		let visit_predicates = visited_subjects.insert(subject).is_none();

		Subject::new(
			self.dataset,
			self.graph,
			subject,
			&visited_subjects,
			self.visited_graphs,
			visit_predicates,
		)
		.visit(&mut visitor)?;

		visitor.end()
	}
}

struct Subject<'a, 'v, D: rdfx::dataset::Dataset>
where
	D: PatternMatchingDataset,
	D::Subject: rdfx::Resource,
{
	dataset: &'a D,
	graph: Option<&'a D::Subject>,
	subject: &'a D::Subject,
	visited_subjects: &'v im::HashSet<&'a D::Subject>,
	visited_graphs: &'v im::HashSet<&'a D::Subject>,
	visit_predicates: bool,
}

impl<'a, 'v, D> Subject<'a, 'v, D>
where
	D: rdfx::dataset::Dataset
		+ SubjectTraversableDataset
		+ PredicateTraversableDataset
		+ PatternMatchingDataset,
	D::Subject: rdfx::Resource + Eq + Hash,
{
	fn new(
		dataset: &'a D,
		graph: Option<&'a D::Subject>,
		subject: &'a D::Subject,
		visited_subjects: &'v im::HashSet<&'a D::Subject>,
		visited_graphs: &'v im::HashSet<&'a D::Subject>,
		visit_predicates: bool,
	) -> Self {
		Self {
			dataset,
			graph,
			subject,
			visited_subjects,
			visited_graphs,
			visit_predicates,
		}
	}

	fn visit<I: Interpretation<Resource = D::Subject>, S>(
		&self,
		visitor: &mut S,
	) -> Result<(), S::Error>
	where
		S: SubjectVisitor<I>,
		D::Subject: LinkedDataResource<I>,
		<D as rdfx::dataset::Dataset>::Graph: PartialEq<D::Subject>,
	{
		if self.visit_predicates {
			for (predicate, _) in self
				.dataset
				.quad_predicates_objects(self.graph, self.subject)
			{
				visitor.predicate(
					predicate,
					&PredicateObjects {
						dataset: self.dataset,
						graph: self.graph,
						subject: self.subject,
						predicate,
						visited_subjects: self.visited_subjects,
						visited_graphs: self.visited_graphs,
					},
				)?;
			}

			if self.dataset.contains_named_graph(self.subject) {
				let mut visited_graphs = self.visited_graphs.clone();
				if visited_graphs.insert(self.subject).is_none() {
					visitor.graph(&NamedGraphView {
						dataset: self.dataset,
						graph: self.subject,
						visited_graphs: &visited_graphs,
					})?;
				}
			}
		}

		Ok(())
	}
}

impl<'a, 'v, I: Interpretation, D> LinkedDataResource<I> for Subject<'a, 'v, D>
where
	I::Resource: LinkedDataResource<I> + rdfx::Resource,
	D: rdfx::dataset::Dataset + Graph<Subject = I::Resource> + PatternMatchingDataset,
{
	fn interpretation(&self, interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		self.subject.interpretation(interpretation)
	}
}

impl<'a, 'v, I: Interpretation, D> LinkedDataSubject<I> for Subject<'a, 'v, D>
where
	I::Resource: Eq + Hash + LinkedDataResource<I> + rdfx::Resource,
	D: rdfx::dataset::Dataset
		+ SubjectTraversableDataset
		+ PredicateTraversableDataset
		+ PatternMatchingDataset
		+ Graph<Subject = I::Resource>,
{
	fn visit_subject<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		for (predicate, _) in self
			.dataset
			.quad_predicates_objects(self.graph, self.subject)
		{
			visitor.predicate(
				predicate,
				&PredicateObjects {
					dataset: self.dataset,
					graph: self.graph,
					subject: self.subject,
					predicate,
					visited_subjects: self.visited_subjects,
					visited_graphs: self.visited_graphs,
				},
			)?;
		}

		visitor.end()
	}
}

struct NamedGraphView<'a, 'v, D: rdfx::dataset::Dataset>
where
	D: PatternMatchingDataset,
	D::Subject: rdfx::Resource,
{
	dataset: &'a D,
	graph: &'a D::Subject,
	visited_graphs: &'v im::HashSet<&'a D::Subject>,
}

impl<'a, 'v, I: Interpretation, D> LinkedDataResource<I> for NamedGraphView<'a, 'v, D>
where
	I::Resource: LinkedDataResource<I> + rdfx::Resource,
	D: rdfx::dataset::Dataset + Graph<Subject = I::Resource> + PatternMatchingDataset,
{
	fn interpretation(&self, interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		self.graph.interpretation(interpretation)
	}
}

impl<'a, 'v, I: Interpretation, D> LinkedDataGraph<I> for NamedGraphView<'a, 'v, D>
where
	I::Resource: Eq + Hash + LinkedDataResource<I> + rdfx::Resource,
	D: rdfx::dataset::Dataset
		+ SubjectTraversableDataset
		+ PredicateTraversableDataset
		+ PatternMatchingDataset
		+ Graph<Subject = I::Resource>,
	<D as rdfx::dataset::Dataset>::Graph: PartialEq<I::Resource>,
{
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: GraphVisitor<I>,
	{
		let mut visited_subjects = im::HashSet::new();
		let mut graph_subjects = Vec::new();

		for subject in self.dataset.subjects() {
			if self
				.dataset
				.quad_pattern_matching(
					Quad(Some(subject), None, None, Some(Some(self.graph))).into(),
				)
				.next()
				.is_some()
			{
				visited_subjects.insert(subject);
				graph_subjects.push(subject);
			}
		}

		for subject in graph_subjects {
			visitor.subject(&Subject::new(
				self.dataset,
				Some(self.graph),
				subject,
				&visited_subjects,
				self.visited_graphs,
				true,
			))?;
		}

		visitor.end()
	}
}
