# chaff

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

An experimental jam/cue (Nock noun serialization) implementation used as an alternative `Jammer` for `NounSlab`, with benchmarks comparing it against the default checkpoint serializers.

## Role in the Workspace

`chaff` is primarily a library crate providing the `Chaff` jammer: it serializes (`jam`) and deserializes (`cue`) Nock nouns to/from byte buffers using a mug-keyed deduplication map and the bit-level reader/writer primitives from the `habit` crate. It implements `nockapp`'s `Jammer` trait so it can be plugged in as a drop-in noun (de)serializer. The crate exists mainly to evaluate serialization performance, so it ships Criterion benchmarks and a standalone benchmark binary that round-trips real `nockapp` checkpoints. It is marked `publish = false`.

## Key Components

- `Chaff` — the jammer type; `Chaff::jam(noun, space) -> Bytes` and `Chaff::cue_into(allocator, bytes) -> Result<Noun, CueError>`
- `impl Jammer for Chaff` — integrates `Chaff` as a `nockapp` `NounSlab` jammer
- `CueError` — `BadBackref`, `BackrefTooBig`, `TruncatedBuffer` decode failures
- `NounMap` — internal mug-keyed map for backreference deduplication during jam/cue
- `bin/large_checkpoint_bench` (`large_checkpoint_bench`) — manual one-shot benchmark that decodes and re-jams large `nockapp` checkpoint envelopes
- `benches/jam_checkpoint` — Criterion benchmark harness

## Usage

```sh
# Manual checkpoint round-trip benchmark
cargo run -p chaff --release --bin large_checkpoint_bench

# Criterion benchmarks
cargo bench -p chaff
```
