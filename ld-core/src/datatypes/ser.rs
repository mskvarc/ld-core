use rdf_rs::Interpretation;

use crate::{
	BorrowedRdfTerm, CowRdfTerm, LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject,
	OwnedRdfTerm, PredicateObjectsVisitor, RdfLiteral, RdfLiteralRef, ResourceInterpretation,
};

macro_rules! datatype {
	($($ty:ty : $variant:ident),*) => {
		$(
			impl<I: Interpretation> LinkedDataResource<I> for $ty {
				fn interpretation(
					&self,
					_interpretation: &mut I,
				) -> ResourceInterpretation<'_, I> {
					ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Owned(OwnedRdfTerm::Literal(RdfLiteral::Xsd(
						xsd_rs::Value::$variant(self.clone())
					)))))
				}
			}

			impl<I: Interpretation> LinkedDataSubject<I> for $ty {
				fn visit_subject<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
				where
					S: crate::SubjectVisitor<I>
				{
					visitor.end()
				}
			}

			impl<I: Interpretation> LinkedDataPredicateObjects<I> for $ty {
				fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
				where
					S: PredicateObjectsVisitor<I>,
				{
					visitor.object(self)?;
					visitor.end()
				}
			}
		)*
	};
}

macro_rules! unsized_datatype {
	($($ty:ty : $variant:ident),*) => {
		$(
			impl<I: Interpretation> LinkedDataResource<I> for $ty {
				fn interpretation(
					&self,
					_interpretation: &mut I,
				) -> ResourceInterpretation<'_, I> {
					ResourceInterpretation::Uninterpreted(Some(CowRdfTerm::Borrowed(BorrowedRdfTerm::Literal(RdfLiteralRef::Xsd(
						xsd_rs::ValueRef::$variant(self)
					)))))
				}
			}

			impl<I: Interpretation> LinkedDataSubject<I> for $ty {
				fn visit_subject<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
				where
					S: crate::SubjectVisitor<I>
				{
					visitor.end()
				}
			}

			impl<I: Interpretation> LinkedDataPredicateObjects<I> for $ty {
				fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
				where
					S: PredicateObjectsVisitor<I>,
				{
					visitor.object(self)?;
					visitor.end()
				}
			}
		)*
	};
}

datatype! {
	u8: UnsignedByte,
	u16: UnsignedShort,
	u32: UnsignedInt,
	u64: UnsignedLong,
	i8: Byte,
	i16: Short,
	i32: Int,
	i64: Long,
	String: String,
	xsd_rs::DateTime: DateTime
}

unsized_datatype! {
	str: String
}
