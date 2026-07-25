# ld-core

[![Crate](https://img.shields.io/crates/v/ld-core.svg?style=flat-square)](https://crates.io/crates/ld-core)
[![Docs](https://img.shields.io/docsrs/ld-core?style=flat-square)](https://docs.rs/ld-core)
[![MSRV](https://img.shields.io/crates/msrv/ld-core?style=flat-square)](https://crates.io/crates/ld-core)
[![License](https://img.shields.io/crates/l/ld-core.svg?style=flat-square)](#license)

Serialize and deserialize Rust types as Linked Data. Traits describing how a value maps onto RDF subjects, predicates and graphs, plus derive macros that write those impls for you.

```rust
use iri_rs::IriBuf;
use ld_core::{Deserialize, Serialize, to_quads};
use ld_core::rdfx::{RdfDisplay, generator};

#[derive(Serialize, Deserialize)]
#[ld(prefix("ex" = "http://example.org/"))]
struct Foo {
    #[ld(id)]
    id: IriBuf,

    #[ld("ex:name")]
    name: String,

    #[ld("ex:email")]
    email: String,
}

let value = Foo {
    id: IriBuf::new("http://example.org/JohnSmith".to_owned()).unwrap(),
    name: "John Smith".to_owned(),
    email: "john.smith@example.org".to_owned(),
};

for quad in to_quads(generator::Blank::new(), &value).unwrap() {
    println!("{} .", quad.rdf_display());
}
```

```text
<http://example.org/JohnSmith> <http://example.org/name> "John Smith" .
<http://example.org/JohnSmith> <http://example.org/email> "john.smith@example.org" .
```

---

## Why this fork

Fork of [`linked-data`](https://crates.io/crates/linked-data) by [Spruce Systems, Inc.](https://github.com/spruceid/linked-data-rs). The traits, the derive attributes and the RDF mapping are unchanged; this fork retargets the crate at a different dependency stack:

- Renamed crates: `linked-data` → `ld-core`, `linked-data-derive` → `ld-core-derive`.
- Depends on [`rdfx`](https://crates.io/crates/rdfx) instead of `rdf-types`, [`xsd-rs`](https://crates.io/crates/xsd-rs) instead of `xsd-types`, [`iri-rs`](https://crates.io/crates/iri-rs) instead of `iref` / `static-iref`, and [`jstrict`](https://crates.io/crates/jstrict) instead of `json-syntax` — which is the point of the fork.
- RDF 1.2 aware through `rdfx`: triple terms and directional language strings pass through the quad conversions.
- Rust 2024 edition, MSRV 1.85 (upstream: 2021, 1.71.1).
- Warning-free build and a reshaped workspace.

Credit and history preserved — see [Attribution](#attribution).

## Install

```sh
cargo add ld-core
```

## Feature flags

| Flag     | Default | Enables                                                          |
| -------- | :-----: | ---------------------------------------------------------------- |
| `derive` |   yes   | The `Serialize` / `Deserialize` derive macros (`ld-core-derive`)  |
| `serde`  |   yes   | `serde` interop, including the `json_literal!` macro             |

## Derive attributes

The derive macros read `#[ld(...)]` attributes:

| Attribute                     | Meaning                                                       |
| ----------------------------- | ------------------------------------------------------------- |
| `#[ld(prefix("ex" = "..."))]` | Declares a prefix usable in later attributes                   |
| `#[ld(type = "...")]`         | Emits an `rdf:type` triple for the subject                     |
| `#[ld(id)]`                   | The field holds the subject IRI                                |
| `#[ld("ex:name")]`            | The field is the object of that predicate                      |
| `#[ld(flatten)]`              | Merges the field's own properties into the parent subject      |
| `#[ld(graph)]`                | The field's quads go into a named graph                        |
| `#[ld(ignore)]`               | The field takes no part in serialization                       |

See `ld-core/examples/` for runnable end-to-end usage:

```sh
cargo run --example derive
cargo run --example derive_enum
cargo run --example generics
cargo run --example graph
```

## Interpretations and vocabularies

`to_quads` produces lexical quads with blank nodes drawn from a generator. When resources are interned — indices from an `rdfx` vocabulary, say — use the interpreted counterparts instead: `to_interpreted_quads`, `to_interpreted_subject_quads` and `to_interpreted_graph_quads`, which keep values in the interpretation domain rather than materialising IRIs.

## MSRV

Rust 1.85 (edition 2024).

## Attribution

Original crates: [`linked-data`](https://crates.io/crates/linked-data) and [`linked-data-derive`](https://crates.io/crates/linked-data-derive) by [Spruce Systems, Inc.](https://github.com/spruceid/linked-data-rs). Upstream commits are preserved in this repo's history under their original authorship. This fork is a dependency swap onto the `rdfx` / `xsd-rs` / `iri-rs` / `jstrict` stack, plus edition 2024 polish.

## License

Dual-licensed, same as upstream. Pick whichever fits:

- [Apache-2.0](https://github.com/mskvarc/ld-core/blob/master/LICENSE-APACHE.md)
- [MIT](https://github.com/mskvarc/ld-core/blob/master/LICENSE-MIT.md)
