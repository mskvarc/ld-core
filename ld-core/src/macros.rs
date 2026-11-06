/// Implements the Linked-Data resource traits for a `serde` type, mapping it
/// onto a canonical `rdf:JSON` literal.
///
/// The type must implement `serde::Serialize` and `serde::Deserialize`. The
/// generated impls serialize the value to JSON, canonicalize it, and store it
/// as a literal of type [`RDF_JSON`](crate::RDF_JSON).
#[macro_export]
macro_rules! json_literal {
	($ty:ty) => {
		impl<I: $crate::rdfx::Interpretation> $crate::LinkedDataResource<I> for $ty {
			fn interpretation(
				&self,
				_interpretation: &mut I,
			) -> $crate::ResourceInterpretation<'_, I> {
				use $crate::{CowRdfTerm, OwnedRdfTerm, RdfLiteral, ResourceInterpretation};

				let mut value = $crate::jstrict::to_value(self).unwrap();
				value.canonicalize();

				ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Owned(
					OwnedRdfTerm::Literal(RdfLiteral::Json(value)),
				)))
			}
		}

		impl<I: $crate::rdfx::Interpretation> $crate::LinkedDataPredicateObjects<I> for $ty {
			fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
			where
				S: $crate::PredicateObjectsVisitor<I>,
			{
				visitor.object(self)?;
				visitor.end()
			}
		}

		impl<I: $crate::rdfx::Interpretation> $crate::LinkedDataDeserializePredicateObjects<I>
			for $ty
		where
			I: $crate::rdfx::interpretation::ReverseInterpretation,
			I::Resource: $crate::rdfx::Resource,
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
				D: $crate::rdfx::dataset::PatternMatchingDataset<Subject = I::Resource>,
			{
				let mut objects = objects.into_iter();
				match objects.next() {
					Some(object) => {
						if objects.next().is_none() {
							let deserialize = <Self as $crate::LinkedDataDeserializeSubject<I>>::deserialize_subject_in;
							deserialize(interpretation, dataset, graph, object, context)
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

		impl<I: $crate::rdfx::Interpretation> $crate::LinkedDataSubject<I> for $ty {
			fn visit_subject<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
			where
				S: $crate::SubjectVisitor<I>,
			{
				visitor.end()
			}
		}

		impl<I: $crate::rdfx::Interpretation> $crate::LinkedDataDeserializeSubject<I> for $ty
		where
			I: $crate::rdfx::interpretation::ReverseInterpretation,
			I::Resource: $crate::rdfx::Resource,
		{
			fn deserialize_subject_in<D>(
				interpretation: &I,
				_dataset: &D,
				_graph: Option<&I::Resource>,
				resource: &I::Resource,
				context: $crate::Context<I>,
			) -> Result<Self, $crate::FromLinkedDataError>
			where
				D: $crate::rdfx::dataset::PatternMatchingDataset<Subject = I::Resource>,
			{
				let mut literal_ty = None;
				for l in $crate::rdfx::interpretation::ReverseInterpretation::literals_of(
					interpretation,
					resource,
				) {
					match l.type_ {
						$crate::rdfx::CowLiteralType::Any(ref ty) => {
							let ty_iri = ty.as_iri();
							if ty_iri.as_str() == $crate::RDF_JSON.as_str() {
								use $crate::jstrict::Parse;
								let (json, _) = $crate::jstrict::Value::parse_str(&l.value)
									.map_err(|_| {
										$crate::FromLinkedDataError::InvalidLiteral(
											context.into_iris(interpretation),
										)
									})?;

								return $crate::jstrict::from_value(json).map_err(|_| {
									$crate::FromLinkedDataError::InvalidLiteral(
										context.into_iris(interpretation),
									)
								});
							} else {
								literal_ty = Some(ty_iri.into())
							}
						}
						$crate::rdfx::CowLiteralType::LangString(_)
						| $crate::rdfx::CowLiteralType::DirLangString { .. } => {
							literal_ty = Some($crate::rdfx::RDF_LANG_STRING.into())
						}
					}
				}

				match literal_ty {
					Some(ty) => Err($crate::FromLinkedDataError::LiteralTypeMismatch {
						context: context.into_iris(interpretation),
						expected: Some(Box::new($crate::RDF_JSON.into())),
						found: Box::new(ty),
					}),
					None => Err($crate::FromLinkedDataError::ExpectedLiteral(
						context.into_iris(interpretation),
					)),
				}
			}
		}
	};
}

#[cfg(all(test, feature = "serde"))]
mod test {
	#[derive(Debug, serde::Serialize, serde::Deserialize)]
	struct Test {
		field: String,
	}

	json_literal!(Test);

	#[test]
	fn json_literal_macro_accepts_a_serde_type() {
		let test = Test {
			field: "value".to_owned(),
		};
		assert_eq!(test.field, "value");
	}
}
