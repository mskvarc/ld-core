use rdfx::{Interpretation, RDF_LANG_STRING, interpretation::ReverseInterpretation};

use crate::{
	Context, FromLinkedDataError, LinkedDataDeserializePredicateObjects,
	LinkedDataDeserializeSubject,
};

macro_rules! deserialize_datatype {
	($($ty:ty : $iri:ident),*) => {
		$(
			impl<I: Interpretation> LinkedDataDeserializeSubject<I> for $ty
			where
				I: ReverseInterpretation,
				I::Resource: rdfx::Resource,
			{
				fn deserialize_subject_in<D>(
					interpretation: &I,
					_dataset: &D,
					_graph: Option<&I::Resource>,
					resource: &I::Resource,
					context: Context<I>
				) -> Result<Self, FromLinkedDataError>
				where
					D: rdfx::dataset::PatternMatchingDataset<Subject = I::Resource>
				{
					let mut literal_ty = None;
					for l in interpretation.literals_of(resource) {
						match &l.type_ {
							rdfx::CowLiteralType::Any(ty) => {
								let ty_iri = ty.as_iri();
								if ty_iri.as_str() == xsd_rs::$iri.as_str() {
									return match l.value.parse() {
										Ok(value) => Ok(value),
										Err(_) => Err(FromLinkedDataError::InvalidLiteral(
											context.into_iris(interpretation)
										))
									}
								}

								literal_ty = Some(ty_iri.into())
							}
							rdfx::CowLiteralType::LangString(_)
							| rdfx::CowLiteralType::DirLangString { .. } => {
								literal_ty = Some(RDF_LANG_STRING.into())
							}
						}
					}

					match literal_ty {
						Some(ty) => {
							Err(FromLinkedDataError::LiteralTypeMismatch {
								context: context.into_iris(interpretation),
								expected: Some(xsd_rs::$iri.into()),
								found: ty
							})
						}
						None => {
							Err(FromLinkedDataError::ExpectedLiteral(
								context.into_iris(interpretation)
							))
						}
					}
				}
			}

			impl<I: Interpretation> LinkedDataDeserializePredicateObjects<I> for $ty
			where
				I: ReverseInterpretation,
				I::Resource: rdfx::Resource,
			{
				fn deserialize_objects_in<'a, D>(
					interpretation: &I,
					dataset: &D,
					graph: Option<&I::Resource>,
					objects: impl IntoIterator<Item = &'a <I as Interpretation>::Resource>,
					context: Context<I>
				) -> Result<Self, FromLinkedDataError>
				where
					<I as Interpretation>::Resource: 'a,
					D: rdfx::dataset::PatternMatchingDataset<Subject = I::Resource>
				{
					let mut error = None;

					for o in objects {
						match Self::deserialize_subject_in(interpretation, dataset, graph, o, context) {
							Ok(value) => return Ok(value),
							Err(e) => error = Some(e)
						}
					}

					Err(error.unwrap_or_else(|| {
						FromLinkedDataError::MissingRequiredValue(
							context.into_iris(interpretation)
						)
					}))
				}
			}
		)*
	};
}

deserialize_datatype!(
	u8: XSD_UNSIGNED_BYTE,
	u16: XSD_UNSIGNED_SHORT,
	u32: XSD_UNSIGNED_INT,
	u64: XSD_UNSIGNED_LONG,
	i8: XSD_BYTE,
	i16: XSD_SHORT,
	i32: XSD_INT,
	i64: XSD_LONG,
	String: XSD_STRING,
	xsd_rs::DateTime: XSD_DATE_TIME
);
