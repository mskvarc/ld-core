#[macro_export]
macro_rules! json_literal {
	($ty:ty) => {
		impl<I: $crate::rdf_rs::Interpretation> $crate::LinkedDataResource<I> for $ty {
			fn interpretation(
				&self,
				_interpretation: &mut I,
			) -> $crate::ResourceInterpretation<'_, I> {
				use $crate::{CowRdfTerm, OwnedRdfTerm, RdfLiteral, ResourceInterpretation};

				let mut value = $crate::json_syntax::to_value(self).unwrap();
				value.canonicalize();

				ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Owned(
					OwnedRdfTerm::Literal(RdfLiteral::Json(value)),
				)))
			}
		}

		impl<I: $crate::rdf_rs::Interpretation> $crate::LinkedDataPredicateObjects<I> for $ty {
			fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
			where
				S: $crate::PredicateObjectsVisitor<I>,
			{
				visitor.object(self)?;
				visitor.end()
			}
		}

		impl<I: $crate::rdf_rs::Interpretation> $crate::LinkedDataDeserializePredicateObjects<I>
			for $ty
		where
			I: $crate::rdf_rs::interpretation::ReverseInterpretation,
			I::Resource: $crate::rdf_rs::Resource,
		{
			fn deserialize_objects_in<'a, D>(
				interpretation: &I,
				dataset: &D,
				graph: Option<&I::Resource>,
				objects: impl IntoIterator<Item = &'a I::Resource>,
				context: $crate::Context<I>,
			) -> Result<Self, $crate::FromLinkedDataError>
			where
				I::Resource: 'a,
				D: $crate::rdf_rs::dataset::PatternMatchingDataset<Subject = I::Resource>,
			{
				let mut objects = objects.into_iter();
				match objects.next() {
					Some(object) => {
						if objects.next().is_none() {
							<Self as $crate::LinkedDataDeserializeSubject<I>>::deserialize_subject_in(
								interpretation, dataset, graph, object, context,
							)
						} else {
							Err($crate::FromLinkedDataError::TooManyValues(
								context.into_iris(interpretation),
							))
						}
					}
					None => Err($crate::FromLinkedDataError::MissingRequiredValue(
						context.into_iris(interpretation),
					)),
				}
			}
		}

		impl<I: $crate::rdf_rs::Interpretation> $crate::LinkedDataSubject<I> for $ty {
			fn visit_subject<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
			where
				S: $crate::SubjectVisitor<I>,
			{
				visitor.end()
			}
		}

		impl<I: $crate::rdf_rs::Interpretation> $crate::LinkedDataDeserializeSubject<I> for $ty
		where
			I: $crate::rdf_rs::interpretation::ReverseInterpretation,
			I::Resource: $crate::rdf_rs::Resource,
		{
			fn deserialize_subject_in<D>(
				interpretation: &I,
				_dataset: &D,
				_graph: Option<&I::Resource>,
				resource: &I::Resource,
				context: $crate::Context<I>,
			) -> Result<Self, $crate::FromLinkedDataError>
			where
				D: $crate::rdf_rs::dataset::PatternMatchingDataset<Subject = I::Resource>,
			{
				let mut literal_ty = None;
				for l in $crate::rdf_rs::interpretation::ReverseInterpretation::literals_of(
					interpretation, resource,
				) {
					match l.type_ {
						$crate::rdf_rs::CowLiteralType::Any(ref ty) => {
							let ty_iri = ty.as_iri();
							if ty_iri.as_str() == $crate::RDF_JSON.as_str() {
								use $crate::json_syntax::Parse;
								let (json, _) = $crate::json_syntax::Value::parse_str(&l.value)
									.map_err(|_| {
										$crate::FromLinkedDataError::InvalidLiteral(
											context.into_iris(interpretation),
										)
									})?;

								return $crate::json_syntax::from_value(json).map_err(|_| {
									$crate::FromLinkedDataError::InvalidLiteral(
										context.into_iris(interpretation),
									)
								});
							} else {
								literal_ty = Some(ty_iri.into())
							}
						}
						$crate::rdf_rs::CowLiteralType::LangString(_)
						| $crate::rdf_rs::CowLiteralType::DirLangString { .. } => {
							literal_ty = Some(
								$crate::rdf_rs::RDF_LANG_STRING.into(),
							)
						}
					}
				}

				match literal_ty {
					Some(ty) => Err($crate::FromLinkedDataError::LiteralTypeMismatch {
						context: context.into_iris(interpretation),
						expected: Some($crate::RDF_JSON.into()),
						found: ty,
					}),
					None => Err($crate::FromLinkedDataError::ExpectedLiteral(
						context.into_iris(interpretation),
					)),
				}
			}
		}
	};
}

#[cfg(test)]
mod test {
	#[derive(Debug, serde::Serialize, serde::Deserialize)]
	struct Test {
		field: String,
	}

	json_literal!(Test);
}
