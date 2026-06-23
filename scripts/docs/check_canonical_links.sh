#!/usr/bin/env sh
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
OPEN_ROOT="$(CDPATH= cd -- "${SCRIPT_DIR}/../.." && pwd)"
cd "$OPEN_ROOT"

# Scope is intentionally narrow: only spine docs that declare protocol authority routes.
TARGET_FILES='
START_HERE.md
ARCHITECTURE.md
WORKFLOWS.md
docs/DECISIONS/README.md
PROTOCOL.md
docs/changelog/protocol/SPECIFICATION.md
'

failures=0

require_link() {
    file="$1"
    link="$2"
    reason="$3"
    if ! rg -F -q "](${link})" "$file"; then
        printf '  - %s: missing canonical link `%s` (%s)\n' "$file" "$link" "$reason"
        return 1
    fi
    return 0
}

forbid_link() {
    file="$1"
    link="$2"
    reason="$3"
    if rg -F -q "](${link})" "$file"; then
        printf '  - %s: found non-canonical link `%s` (%s)\n' "$file" "$link" "$reason"
        return 1
    fi
    return 0
}

for file in $TARGET_FILES; do
    if [ ! -f "$file" ]; then
        printf '  - %s: file not found\n' "$file"
        failures=1
    fi
done

if [ "$failures" -ne 0 ]; then
    printf 'canonical protocol-link check: FAIL\n'
    exit 1
fi

if ! require_link "START_HERE.md" "./PROTOCOL.md" "Tier-0 protocol authority must route through open/PROTOCOL.md"; then
    failures=1
fi
if ! rg -q '^## Promotion Gate \(Mandatory\)$' "START_HERE.md"; then
    printf '  - START_HERE.md: missing required section `## Promotion Gate (Mandatory)`\n'
    failures=1
fi
if ! forbid_link "START_HERE.md" "../PROTOCOL.md" "wrong relative path from open root"; then
    failures=1
fi
if ! forbid_link "START_HERE.md" "../../PROTOCOL.md" "wrong relative path from open root"; then
    failures=1
fi

if ! require_link "ARCHITECTURE.md" "./PROTOCOL.md" "architecture authority must reference protocol index"; then
    failures=1
fi
if ! require_link "ARCHITECTURE.md" "./docs/changelog/protocol/SPECIFICATION.md" "architecture points to protocol schema authority"; then
    failures=1
fi
if ! forbid_link "ARCHITECTURE.md" "../PROTOCOL.md" "wrong relative path from open root"; then
    failures=1
fi
if ! forbid_link "ARCHITECTURE.md" "../../PROTOCOL.md" "wrong relative path from open root"; then
    failures=1
fi
if ! forbid_link "ARCHITECTURE.md" "../changelog/protocol/SPECIFICATION.md" "points outside open lane"; then
    failures=1
fi
if ! forbid_link "ARCHITECTURE.md" "../../changelog/protocol/SPECIFICATION.md" "points outside open lane"; then
    failures=1
fi

if ! require_link "WORKFLOWS.md" "./PROTOCOL.md" "workflow route for upgrades must use protocol index"; then
    failures=1
fi
if ! forbid_link "WORKFLOWS.md" "../PROTOCOL.md" "wrong relative path from open root"; then
    failures=1
fi
if ! forbid_link "WORKFLOWS.md" "../../PROTOCOL.md" "wrong relative path from open root"; then
    failures=1
fi

if ! require_link "docs/DECISIONS/README.md" "../../PROTOCOL.md" "ADR index is two directories below repo root"; then
    failures=1
fi
if ! forbid_link "docs/DECISIONS/README.md" "./PROTOCOL.md" "wrong relative path from docs/DECISIONS/"; then
    failures=1
fi
if ! forbid_link "docs/DECISIONS/README.md" "../PROTOCOL.md" "wrong relative path from docs/DECISIONS/"; then
    failures=1
fi

if ! require_link "docs/changelog/protocol/SPECIFICATION.md" "../../../PROTOCOL.md" "specification schema must point to protocol index"; then
    failures=1
fi
if ! forbid_link "docs/changelog/protocol/SPECIFICATION.md" "./PROTOCOL.md" "wrong relative path from docs/changelog/protocol/"; then
    failures=1
fi
if ! forbid_link "docs/changelog/protocol/SPECIFICATION.md" "../../PROTOCOL.md" "wrong relative path from docs/changelog/protocol/"; then
    failures=1
fi

if ! require_link "PROTOCOL.md" "./docs/changelog/protocol/SPECIFICATION.md" "protocol index must include schema authority"; then
    failures=1
fi
if ! rg -q '\]\(\./docs/changelog/protocol/[0-9]{3}[-a-z0-9]*\.md\)' "PROTOCOL.md"; then
    printf '  - PROTOCOL.md: missing versioned upgrade links under `./docs/changelog/protocol/`\n'
    failures=1
fi
if ! forbid_link "PROTOCOL.md" "../changelog/protocol/SPECIFICATION.md" "points outside open lane"; then
    failures=1
fi
if ! forbid_link "PROTOCOL.md" "../../changelog/protocol/SPECIFICATION.md" "points outside open lane"; then
    failures=1
fi

if [ "$failures" -ne 0 ]; then
    printf 'canonical protocol-link check: FAIL\n'
    exit 1
fi

printf 'canonical protocol-link check: PASS\n'
