# Transaction Engine: Architecture Deep Dives

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (explanatory deep dives; protocol authority remains [`PROTOCOL.md`](../../../PROTOCOL.md))

A guided, in-depth walkthrough of how the Nockchain transaction engine works:
the UTXO/note model, the witness and lock machinery, validation, and the
cryptographic and data-structure primitives underneath it.

> These are explanatory references, not normative protocol rules. When this
> material conflicts with [`PROTOCOL.md`](../../../PROTOCOL.md) or the upgrade
> specs in [`changelog/protocol/`](../../../changelog/protocol/), the protocol
> docs win.

## Read Order

Start with the overview, then follow the numbered sequence.

| #  | Document                                                                    | Topic                                                        |
| -- | --------------------------------------------------------------------------- | ----------------------------------------------------------- |
| 00 | [`00-overview.md`](./00-overview.md)                                         | Architecture overview and how the pieces fit together       |
| 01 | [`01-utxo-model.md`](./01-utxo-model.md)                                     | UTXO model: notes, names, and balances                      |
| 02 | [`02-segwit-witness-separation.md`](./02-segwit-witness-separation.md)       | SegWit-inspired witness separation                          |
| 03 | [`03-taproot-lock-merkle-proofs.md`](./03-taproot-lock-merkle-proofs.md)     | Taproot-inspired lock Merkle proofs (MAST)                  |
| 04 | [`04-eutxo-note-data.md`](./04-eutxo-note-data.md)                           | Extended UTXO: note data as on-chain datum                  |
| 05 | [`05-lock-primitives-script-model.md`](./05-lock-primitives-script-model.md) | Lock primitives: a composable script model                  |
| 06 | [`06-fee-structure.md`](./06-fee-structure.md)                               | Fee structure: weight discounting                           |
| 07 | [`07-transaction-validation-pipeline.md`](./07-transaction-validation-pipeline.md) | Transaction validation pipeline                       |
| 08 | [`08-protocol-evolution.md`](./08-protocol-evolution.md)                     | Protocol evolution: upgrade mechanics and history           |
| 09 | [`09-noun-encoding-data-layer.md`](./09-noun-encoding-data-layer.md)         | Noun encoding and the Nock data layer                       |
| 10 | [`10-zoon-persistent-data-structures.md`](./10-zoon-persistent-data-structures.md) | Zoon: hash-ordered persistent trees                   |
| 11 | [`11-tip5-hash-function.md`](./11-tip5-hash-function.md)                     | Tip5: the sponge hash function                              |
| 12 | [`12-schnorr-signatures-cheetah-curve.md`](./12-schnorr-signatures-cheetah-curve.md) | Schnorr signatures over the Cheetah curve           |
| 13 | [`13-merkle-trees-and-commitments.md`](./13-merkle-trees-and-commitments.md) | Merkle trees and commitment schemes                         |
| 14 | [`14-goldilocks-field-arithmetic.md`](./14-goldilocks-field-arithmetic.md)   | Goldilocks field arithmetic (ztd one and two)               |
| 15 | [`15-ztd-stark-proof-stack.md`](./15-ztd-stark-proof-stack.md)               | The ZTD STARK proof stack (ztd three through eight)         |

## Related Code

- Shared transaction types: [`crates/nockchain-types/README.md`](../../../crates/nockchain-types/README.md)
- Field/crypto math primitives: [`crates/nockchain-math/README.md`](../../../crates/nockchain-math/README.md)
- ZK jets and proof forms: [`crates/zkvm-jetpack/README.md`](../../../crates/zkvm-jetpack/README.md)
- Wallet transaction planning: [`crates/wallet-tx-builder/README.md`](../../../crates/wallet-tx-builder/README.md)
