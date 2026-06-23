# DOC_INVENTORY

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (coverage tracker for `**/*.md`; authority remains with Tier-0 docs)

Documentation coverage tracker. For navigation, use the documentation map at
[`docs/README.md`](./README.md); this file tracks *coverage*, not routing.

- Source command: `find . -name '*.md' -not -path './.git/*' -not -path './target/*' | sort`
- Last counted: 2026-06-23
- Total tracked markdown files: **134**

## Coverage Summary

| Area                                              | Files | Notes                                                                 |
| ------------------------------------------------- | ----- | -------------------------------------------------------------------- |
| Tier-0 canonical spine (root + `docs/DECISIONS/README`) | 5     | `START_HERE`, `PROTOCOL`, `ARCHITECTURE`, `WORKFLOWS`, `docs/DECISIONS/README` |
| Root quickstart                                   | 1     | `README.md` (Tier-1 quickstart)                                     |
| Decision records and template                     | 3     | `docs/DECISIONS/0001`, `0002`, `TEMPLATE`                               |
| Protocol upgrade specs (`docs/changelog/protocol/`)    | 16    | `SPECIFICATION.md` + 15 upgrade specs (001–014, incl. Aletheia audit) |
| `docs/` (map, inventory, architecture, pma, fixes) | 33    | documentation map, this tracker, `pma/PMA-FAQ.md`, deep dives        |
| `crates/` (module READMEs + subsystem docs)       | 74    | every crate now has a root README; includes bridge/nockvm doc sets   |
| `tests/`                                          | 2     | e2e fixtures/readmes                                                  |

Counts include third-party vendored docs under `crates/nockvm/rust/` (ibig,
murmur3), which are not maintained by this project.

## Module README Coverage

As of 2026-06-23, **every crate under `crates/` has a root `README.md`**. The
22 previously-undocumented crates were given module READMEs following the
crate-level house style (metadata header + `Canonical/Legacy: Legacy`). Browse
them through the [documentation map](./README.md#5-modules-by-subsystem).

## Canonical Tiers

Canonical tiering is governed by [`START_HERE.md`](../START_HERE.md), not by this
file. In summary:

- **Tier 0 (canonical spine):** `START_HERE.md`, `PROTOCOL.md`,
  `ARCHITECTURE.md`, `WORKFLOWS.md`, `docs/DECISIONS/README.md`, plus the protocol
  upgrade specs indexed by `PROTOCOL.md`.
- **Tier 1 (scoped canonical satellites):** `README.md`,
  `crates/nockapp/README.md`, `crates/nockchain-api/README.md`,
  `crates/nockchain-wallet/README.md`.
- **Tier 2 (legacy / contextual):** everything else, including all module
  READMEs and the explanatory deep dives under `docs/`.

## Maintenance

When adding or removing markdown files:

1. Re-run the source command and update the total + area counts above.
2. Add navigation entries to [`docs/README.md`](./README.md).
3. If the doc changes canonical tiering, update [`START_HERE.md`](../START_HERE.md)
   in the same change and run `make docs-check`.
