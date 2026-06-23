# noun-serde

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

Serialization framework for converting Rust types to and from nockvm `Noun` values, providing the `NounEncode`/`NounDecode` traits and re-exporting the matching derive macros.

## Role in the Workspace

This crate defines the encode/decode contract used throughout the workspace to move data between Rust and the Nock runtime. Types implement `NounEncode`/`NounDecode` (by hand or via the derive macros from `noun-serde-derive`, which this crate re-exports) and convert against an allocator and `NounSpace`. It depends on `nockvm` for the noun representation and is used by networking, gRPC, and node crates that need to serialize structured data as nouns.

## Key Components

- `NounEncode` — trait with `to_noun<A: NounAllocator>(&self, allocator)` for encoding a value as a `Noun`.
- `NounDecode` — trait with `from_noun(noun, space)` (and `from_noun_handle`) for decoding a value from a `Noun`.
- `NounSerdeEncodeExt` / `NounSerdeDecodeExt` — convenience extension traits adding `encode`/`decode` methods to `Noun`.
- `NounDecodeError` — error enum covering atom/cell mismatches, field errors, invalid enum variants/tags, and custom and Mary/FPoly decode failures.
- `prelude` — re-exports the core traits and extension traits.
- `NounEncode` / `NounDecode` derives — re-exported from `noun-serde-derive`.
- `wallet` — concrete `NounEncode`/`NounDecode` implementations for wallet types (`Key`, `Coil`, `Meta`, etc.).
