# habit

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

Low-level little-endian bit reader and writer primitives over byte buffers, used by the noun serialization code.

## Role in the Workspace

`habit` is a small, dependency-light library crate (its only runtime dependency is `bytes`) that provides sequential, bit-granular I/O over immutable and growable byte buffers. It is consumed by the `chaff` jam/cue implementation, which builds Nock noun serialization on top of these primitives. The crate is marked `publish = false`.

## Key Components

- `BitReader` — sequential little-endian bit reader over an immutable `Bytes` buffer; `read_bit`, `read_bits_to_u64`/`read_bits_to_usize`, `read_bits_to_bytes`, plus `position` and `bits_remaining`
- `BitWriter` — accumulating bit writer that produces a byte buffer
