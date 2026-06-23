# noun-serde-derive

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

Procedural macro crate that derives the `NounEncode` and `NounDecode` trait implementations used by `noun-serde`.

## Role in the Workspace

This is a `proc-macro` crate (it exports nothing else). It generates `noun-serde`'s encode/decode implementations for structs and enums so callers can `#[derive(NounEncode, NounDecode)]` instead of writing noun (de)serialization by hand. It is a dependency of `noun-serde`, which re-exports these derives; downstream crates normally pull them in through `noun-serde` rather than depending on this crate directly.

## Key Components

- `#[proc_macro_derive(NounEncode, attributes(noun))]` — `derive_noun_encode` generates a `NounEncode` impl.
- `#[proc_macro_derive(NounDecode, attributes(noun))]` — `derive_noun_decode` generates a `NounDecode` impl.
- `#[noun(...)]` attribute — controls enum encoding via `tagged`/`untagged` (whether variant tags are emitted) and `tag = "..."`/`tag = <int>` (explicit text or numeric variant tags). Text tags up to 8 bytes are packed into atom values.

Tagged enum encoding: `[%variant [%variant1 value1] [%variant2 value2] ...]`; untagged encoding: `[%variant value1 value2 ...]`.
