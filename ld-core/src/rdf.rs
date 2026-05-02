use core::fmt;
use std::borrow::Borrow;

use iri_rs::{Iri, IriBuf, iri};
use rdf_rs::{
	BlankId, BlankIdBuf, Datatype, DatatypeRef, Id, Interpretation, InterpretationMut,
	Literal, LiteralType, LiteralTypeRef, LocalTerm, LocalTermRef, Quad,
};

pub type RdfId = Id;
pub type RdfTerm = LocalTerm;
pub type RdfTermRef<'a> = LocalTermRef<'a>;

pub type RdfQuad = Quad<Id, IriBuf, LocalTerm, Id>;

pub type InterpretedQuad<I> = Quad<
	<I as Interpretation>::Resource,
	<I as Interpretation>::Resource,
	<I as Interpretation>::Resource,
	<I as Interpretation>::Resource,
>;

#[derive(Debug, Clone)]
pub enum CowRdfTerm<'a> {
	Borrowed(BorrowedRdfTerm<'a>),
	Owned(OwnedRdfTerm),
}

#[derive(Debug, Clone, Copy)]
pub enum BorrowedRdfTerm<'a> {
	Iri(Iri<&'a str>),
	BlankId(&'a BlankId),
	Literal(RdfLiteralRef<'a>),
}

#[derive(Debug, Clone)]
pub enum OwnedRdfTerm {
	Iri(IriBuf),
	BlankId(BlankIdBuf),
	Literal(RdfLiteral),
}

impl OwnedRdfTerm {
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

pub enum CowRef<'a, T> {
	Borrowed(&'a T),
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

pub trait RdfLiteralValue:
	From<String> + From<xsd_rs::Value> + From<jstrict::Value>
{
}

impl<T: From<String> + From<xsd_rs::Value> + From<jstrict::Value>> RdfLiteralValue for T {}

pub trait AsRdfLiteral {
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
pub enum RdfLiteral {
	Any(String, LiteralType),
	Xsd(xsd_rs::Value),
	Json(jstrict::Value),
}

impl RdfLiteral {
	pub fn into_lexical(self) -> Literal {
		match self {
			Self::Any(s, ty) => Literal::new(s, ty),
			Self::Xsd(value) => {
				let ty = value.datatype().iri().to_owned();
				Literal::new(value.to_string(), datatype_or_unchecked(ty.into()))
			}
			Self::Json(mut value) => {
				value.canonicalize();
				Literal::new(
					value.to_string(),
					datatype_or_unchecked(RDF_JSON.into()),
				)
			}
		}
	}

	pub fn as_literal_ref(&self) -> RdfLiteralRef<'_> {
		match self {
			Self::Any(value, ty) => RdfLiteralRef::Any(value, ty.as_ref()),
			Self::Xsd(value) => RdfLiteralRef::Xsd(value.as_ref()),
			Self::Json(value) => RdfLiteralRef::Json(value),
		}
	}

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

pub const RDF_JSON: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON");

#[derive(Debug, Clone, Copy)]
pub enum RdfLiteralRef<'a> {
	Any(&'a str, LiteralTypeRef<'a>),
	Xsd(xsd_rs::ValueRef<'a>),
	Json(&'a jstrict::Value),
}

impl<'a> RdfLiteralRef<'a> {
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
				Literal::new(
					value.to_string(),
					datatype_or_unchecked(RDF_JSON.into()),
				)
			}
		}
	}

	pub fn into_owned(self) -> RdfLiteral {
		match self {
			Self::Any(value, ty) => RdfLiteral::Any(value.to_owned(), ty.into_owned()),
			Self::Xsd(value) => RdfLiteral::Xsd(value.into_owned()),
			Self::Json(value) => RdfLiteral::Json(value.clone()),
		}
	}
}

pub enum CowRdfLiteral<'a> {
	Borrowed(RdfLiteralRef<'a>),
	Owned(RdfLiteral),
}

impl<'a> CowRdfLiteral<'a> {
	pub fn into_owned(self) -> RdfLiteral {
		match self {
			Self::Borrowed(l) => l.into_owned(),
			Self::Owned(l) => l,
		}
	}

	pub fn as_literal_ref(&self) -> RdfLiteralRef<'_> {
		match self {
			Self::Borrowed(l) => *l,
			Self::Owned(l) => l.as_literal_ref(),
		}
	}
}
