use rdf_rs::{
	dataset::{DatasetGraphView, Graph, PatternMatchingDataset, PredicateTraversableDataset},
	Interpretation,
};
use std::hash::Hash;

use crate::{
	LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject, PredicateObjectsVisitor,
	ResourceInterpretation, SubjectVisitor,
};

impl<'a, I: Interpretation, D> LinkedDataSubject<I> for DatasetGraphView<'a, D>
where
	I::Resource: Eq + Hash + LinkedDataResource<I> + rdf_rs::Resource,
	D: rdf_rs::dataset::Dataset
		+ PredicateTraversableDataset
		+ PatternMatchingDataset
		+ Graph<Subject = I::Resource>,
{
	fn visit_subject<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		let mut visited = im::HashSet::new();
		visited.insert(self.resource);

		Subject::new(self.dataset, self.graph, self.resource, &visited, true)
			.visit(&mut serializer)?;
		serializer.end()
	}
}

struct PredicateObjects<'d, 'v, D: rdf_rs::dataset::Dataset>
where
	D: PatternMatchingDataset,
	D::Subject: rdf_rs::Resource,
{
	dataset: &'d D,
	graph: Option<&'d D::Subject>,
	subject: &'d D::Subject,
	predicate: &'d D::Subject,
	visited: &'v im::HashSet<&'d D::Subject>,
}

impl<'d, 'v, I: Interpretation, D> LinkedDataPredicateObjects<I> for PredicateObjects<'d, 'v, D>
where
	I::Resource: Eq + Hash + LinkedDataResource<I> + rdf_rs::Resource,
	D: rdf_rs::dataset::Dataset
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
				visited: self.visited,
			})?;
		}

		visitor.end()
	}
}

impl<'a, 'v, I: Interpretation, D> LinkedDataResource<I> for Object<'a, 'v, D>
where
	I::Resource: LinkedDataResource<I> + rdf_rs::Resource,
	D: rdf_rs::dataset::Dataset + Graph<Subject = I::Resource> + PatternMatchingDataset,
{
	fn interpretation(&self, interpretation: &mut I) -> ResourceInterpretation<'_, I> {
		self.object.interpretation(interpretation)
	}
}

struct Object<'d, 'v, D: rdf_rs::dataset::Dataset>
where
	D: PatternMatchingDataset,
	D::Subject: rdf_rs::Resource,
{
	dataset: &'d D,
	graph: Option<&'d D::Subject>,
	object: &'d D::Subject,
	visited: &'v im::HashSet<&'d D::Subject>,
}

impl<'d, 'v, I: Interpretation, D> LinkedDataSubject<I> for Object<'d, 'v, D>
where
	I::Resource: Eq + Hash + LinkedDataResource<I> + rdf_rs::Resource,
	D: rdf_rs::dataset::Dataset
		+ PredicateTraversableDataset
		+ PatternMatchingDataset
		+ Graph<Subject = I::Resource>,
{
	fn visit_subject<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
	{
		let subject = self.object;
		let mut visited = self.visited.clone();
		let visit_predicates = visited.insert(subject).is_none();

		Subject::new(
			self.dataset,
			self.graph,
			subject,
			&visited,
			visit_predicates,
		)
		.visit(&mut visitor)?;

		visitor.end()
	}
}

struct Subject<'d, 'v, D: rdf_rs::dataset::Dataset>
where
	D: PatternMatchingDataset,
	D::Subject: rdf_rs::Resource,
{
	dataset: &'d D,
	graph: Option<&'d D::Subject>,
	subject: &'d D::Subject,
	visited: &'v im::HashSet<&'d D::Subject>,
	visit_predicates: bool,
}

impl<'d, 'v, D> Subject<'d, 'v, D>
where
	D: rdf_rs::dataset::Dataset + PredicateTraversableDataset + PatternMatchingDataset,
	D::Subject: rdf_rs::Resource,
{
	fn new(
		dataset: &'d D,
		graph: Option<&'d D::Subject>,
		subject: &'d D::Subject,
		visited: &'v im::HashSet<&'d D::Subject>,
		visit_predicates: bool,
	) -> Self {
		Self {
			dataset,
			graph,
			subject,
			visited,
			visit_predicates,
		}
	}

	fn visit<I: Interpretation<Resource = D::Subject>, S>(&self, visitor: &mut S) -> Result<(), S::Error>
	where
		S: SubjectVisitor<I>,
		I::Resource: Eq + Hash + LinkedDataResource<I> + rdf_rs::Resource,
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
					visited: self.visited,
				},
			)?;
		}

		Ok(())
	}
}

impl<'d, 'v, I: Interpretation, D> LinkedDataSubject<I> for Subject<'d, 'v, D>
where
	D::Subject: Eq + Hash + LinkedDataResource<I> + rdf_rs::Resource,
	D: rdf_rs::dataset::Dataset
		+ PredicateTraversableDataset
		+ PatternMatchingDataset
		+ Graph<Subject = I::Resource>,
{
	fn visit_subject<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: SubjectVisitor<I>,
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
						visited: self.visited,
					},
				)?;
			}
		}

		visitor.end()
	}
}
