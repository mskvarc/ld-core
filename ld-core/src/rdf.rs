use core::fmt;
use std::borrow::Borrow;

use iri_rs::{Iri, IriBuf, iri};
use rdfx::{
	BlankId, BlankIdBuf, Datatype, DatatypeRef, Id, Interpretation, InterpretationMut, Literal,
	LiteralType, LiteralTypeRef, LocalTerm, LocalTermRef, Quad,
};

/// Node identifier: an IRI or a blank node identifier.
pub type RdfId = Id;
/// Lexical RDF term.
pub type RdfTerm = LocalTerm;
/// Borrowed lexical RDF term.
pub type RdfTermRef<'a> = LocalTermRef<'a>;

/// Lexical RDF quad.
pub type RdfQuad = Quad<Id, IriBuf, LocalTerm, Id>;

/// RDF quad whose components are resources of the interpretation `I`.
pub type InterpretedQuad<I> = Quad<
	<I as Interpretation>::Resource,
	<I as Interpretation>::Resource,
	<I as Interpretation>::Resource,
	<I as Interpretation>::Resource,
>;

#[derive(Debug, Clone)]
/// RDF term that is either borrowed or owned.
pub enum CowRdfTerm<'a> {
	/// A borrowed term.
	Borrowed(BorrowedRdfTerm<'a>),
	/// An owned term.
	Owned(OwnedRdfTerm),
}

#[derive(Debug, Clone, Copy)]
/// Borrowed RDF term.
pub enum BorrowedRdfTerm<'a> {
	/// An IRI.
	Iri(Iri<&'a str>),
	/// A blank node identifier.
	BlankId(&'a BlankId),
	/// A literal.
	Literal(RdfLiteralRef<'a>),
}

#[derive(Debug, Clone)]
/// Owned RDF term.
pub enum OwnedRdfTerm {
	/// An IRI.
	Iri(IriBuf),
	/// A blank node identifier.
	BlankId(BlankIdBuf),
	/// A literal.
	Literal(RdfLiteral),
}

impl OwnedRdfTerm {
	/// Converts this term into a node identifier, unless it is a literal.
	pub fn into_id(self) -> Option<Id> {
		match self {
			Self::Iri(iri) => Some(Id::Iri(iri)),
			Self::BlankId(b) => Some(Id::BlankId(b)),
			Self::Literal(_) => None,
		}
	}
}

fn datatype_ref_or_unchecked(ty: Iri<&str>) -> DatatypeRef<'_> {
	DatatypeRef::new(ty).unwrap_or_else(|_| unsafe { DatatypeRef::new_unchecked(ty) })
}

fn datatype_or_unchecked(ty: IriBuf) -> Datatype {
	Datatype::new(ty.clone()).unwrap_or_else(|_| unsafe { Datatype::new_unchecked(ty) })
}

impl<'a> CowRdfTerm<'a> {
	/// Creates a borrowed literal term from a lexical value and its datatype.
	pub fn from_str(value: &'a str, ty: Iri<&'a str>) -> Self {
		use xsd_rs::ValueRef;

		if ty.as_str() == RDF_JSON.as_str() {
			use jstrict::Parse;
			match jstrict::Value::parse_str(value) {
				Ok((json, _)) => CowRdfTerm::Owned(OwnedRdfTerm::Literal(RdfLiteral::Json(json))),
				Err(_) => CowRdfTerm::Borrowed(BorrowedRdfTerm::Literal(RdfLiteralRef::Any(
					value,
					LiteralTypeRef::Any(datatype_ref_or_unchecked(ty)),
				))),
			}
		} else {
			match xsd_rs::Datatype::from_iri(&ty) {
				Some(xsd_rs::Datatype::String(xsd_rs::StringDatatype::String)) => {
					CowRdfTerm::Borrowed(BorrowedRdfTerm::Literal(RdfLiteralRef::Xsd(
						ValueRef::String(value),
					)))
				}
				Some(xsd_ty) => match xsd_ty.parse(value) {
					Ok(xsd_value) => {
						CowRdfTerm::Owned(OwnedRdfTerm::Literal(RdfLiteral::Xsd(xsd_value)))
					}
					Err(_) => CowRdfTerm::Borrowed(BorrowedRdfTerm::Literal(RdfLiteralRef::Any(
						value,
						LiteralTypeRef::Any(datatype_ref_or_unchecked(ty)),
					))),
				},
				None => CowRdfTerm::Borrowed(BorrowedRdfTerm::Literal(RdfLiteralRef::Any(
					value,
					LiteralTypeRef::Any(datatype_ref_or_unchecked(ty)),
				))),
			}
		}
	}

	/// Clones this term into an owned one.
	pub fn into_owned(self) -> OwnedRdfTerm {
		match self {
			Self::Borrowed(b) => match b {
				BorrowedRdfTerm::Iri(i) => OwnedRdfTerm::Iri(i.into()),
				BorrowedRdfTerm::BlankId(b) => OwnedRdfTerm::BlankId(b.to_owned()),
				BorrowedRdfTerm::Literal(l) => OwnedRdfTerm::Literal(l.into_owned()),
			},
			Self::Owned(o) => o,
		}
	}
}

/// Value that is either borrowed or owned, without requiring [`ToOwned`].
pub enum CowRef<'a, T> {
	/// A borrowed value.
	Borrowed(&'a T),
	/// An owned value.
	Owned(T),
}

impl<'a, T> AsRef<T> for CowRef<'a, T> {
	fn as_ref(&self) -> &T {
		match self {
			Self::Borrowed(t) => t,
			Self::Owned(t) => t,
		}
	}
}

impl<'a, T> Borrow<T> for CowRef<'a, T> {
	fn borrow(&self) -> &T {
		match self {
			Self::Borrowed(t) => t,
			Self::Owned(t) => t,
		}
	}
}

/// Value that can be stored as an RDF literal.
pub trait RdfLiteralValue: From<String> + From<xsd_rs::Value> + From<jstrict::Value> {}

impl<T: From<String> + From<xsd_rs::Value> + From<jstrict::Value>> RdfLiteralValue for T {}

/// Type that can be borrowed as an RDF literal of a given datatype.
pub trait AsRdfLiteral {
	/// Borrows this value as an RDF literal of type `ty`.
	fn as_rdf_literal<'a>(&'a self, ty: LiteralTypeRef<'a>) -> CowRdfLiteral<'a>;
}

impl AsRdfLiteral for str {
	fn as_rdf_literal<'a>(&'a self, ty: LiteralTypeRef<'a>) -> CowRdfLiteral<'a> {
		CowRdfLiteral::Borrowed(RdfLiteralRef::Any(self, ty))
	}
}

impl AsRdfLiteral for String {
	fn as_rdf_literal<'a>(&'a self, ty: LiteralTypeRef<'a>) -> CowRdfLiteral<'a> {
		self.as_str().as_rdf_literal(ty)
	}
}

#[derive(Debug, Clone)]
/// Owned RDF literal, kept in its most specific known form.
pub enum RdfLiteral {
	/// Literal held as a lexical value and a datatype.
	Any(String, LiteralType),
	/// Literal held as a parsed XSD value.
	Xsd(xsd_rs::Value),
	/// Literal held as a parsed `rdf:JSON` value.
	Json(jstrict::Value),
}

impl RdfLiteral {
	/// Converts this literal into its lexical form.
	pub fn into_lexical(self) -> Literal {
		match self {
			Self::Any(s, ty) => Literal::new(s, ty),
			Self::Xsd(value) => {
				let ty = value.datatype().iri().to_owned();
				Literal::new(value.to_string(), datatype_or_unchecked(ty.into()))
			}
			Self::Json(mut value) => {
				value.canonicalize();
				Literal::new(value.to_string(), datatype_or_unchecked(RDF_JSON.into()))
			}
		}
	}

	/// Borrows this literal.
	pub fn as_literal_ref(&self) -> RdfLiteralRef<'_> {
		match self {
			Self::Any(value, ty) => RdfLiteralRef::Any(value, ty.as_ref()),
			Self::Xsd(value) => RdfLiteralRef::Xsd(value.as_ref()),
			Self::Json(value) => RdfLiteralRef::Json(value),
		}
	}

	/// Inserts this literal into `interpretation`, returning its resource.
	pub fn embed_into<I: InterpretationMut>(self, interpretation: &mut I) -> I::Resource {
		interpretation.insert_literal(self.into_lexical().into_cow())
	}
}

impl fmt::Display for RdfLiteral {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Any(v, _) => v.fmt(f),
			Self::Xsd(v) => v.fmt(f),
			Self::Json(v) => v.fmt(f),
		}
	}
}

/// The `rdf:JSON` datatype IRI,
/// <http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON>.
pub const RDF_JSON: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON");

#[derive(Debug, Clone, Copy)]
/// Borrowed RDF literal, kept in its most specific known form.
pub enum RdfLiteralRef<'a> {
	/// Literal held as a lexical value and a datatype.
	Any(&'a str, LiteralTypeRef<'a>),
	/// Literal held as a parsed XSD value.
	Xsd(xsd_rs::ValueRef<'a>),
	/// Literal held as a parsed `rdf:JSON` value.
	Json(&'a jstrict::Value),
}

impl<'a> RdfLiteralRef<'a> {
	/// Converts this literal into its lexical form.
	pub fn into_lexical(self) -> Literal {
		match self {
			Self::Any(s, ty) => Literal::new(s.to_owned(), ty.into_owned()),
			Self::Xsd(value) => {
				let ty = value.datatype().iri().to_owned();
				Literal::new(value.to_string(), datatype_or_unchecked(ty.into()))
			}
			Self::Json(value) => {
				let mut value = value.clone();
				value.canonicalize();
				Literal::new(value.to_string(), datatype_or_unchecked(RDF_JSON.into()))
			}
		}
	}

	/// Clones this literal into an owned one.
	pub fn into_owned(self) -> RdfLiteral {
		match self {
			Self::Any(value, ty) => RdfLiteral::Any(value.to_owned(), ty.into_owned()),
			Self::Xsd(value) => RdfLiteral::Xsd(value.into_owned()),
			Self::Json(value) => RdfLiteral::Json(value.clone()),
		}
	}
}

/// RDF literal that is either borrowed or owned.
pub enum CowRdfLiteral<'a> {
	/// A borrowed literal.
	Borrowed(RdfLiteralRef<'a>),
	/// An owned literal.
	Owned(RdfLiteral),
}

impl<'a> CowRdfLiteral<'a> {
	/// Clones this literal into an owned one.
	pub fn into_owned(self) -> RdfLiteral {
		match self {
			Self::Borrowed(l) => l.into_owned(),
			Self::Owned(l) => l,
		}
	}

	/// Borrows this literal.
	pub fn as_literal_ref(&self) -> RdfLiteralRef<'_> {
		match self {
			Self::Borrowed(l) => *l,
			Self::Owned(l) => l.as_literal_ref(),
		}
	}
}
