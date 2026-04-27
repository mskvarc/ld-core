use iri_rs::Iri;
use iri_rs::iri;
use ld_core::{Deserialize, LinkedDataResource, Serialize};

#[derive(Serialize, Deserialize)]
enum Foo<'a> {
	Reference(#[ld(id)] Iri<&'a str>),
}

fn main() {
	println!();
	let a = Foo::Reference(iri!("http://example.org/foo"));
	let mut interpretation = ld_core::rdf_rs::generator::LocalGeneratorInterpretation::new(
		ld_core::rdf_rs::generator::Blank::new(),
	);
	println!("{:?}", a.interpretation(&mut interpretation));
}
