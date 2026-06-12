build:
    cargo build --release

test:
    cargo nextest run --release

test-honk:
    cargo nextest run --release -p honk

build-honk-assets: honc-cold-138-asset hoonc-octs-type-138-asset

honc-cold-138-asset:
    mkdir -p assets target/honk-assets
    target/release/honk --new --dump-wrapper-assets target/honk-assets/wrapper-assets --prelude hoon/common/hoon.hoon hoon
    cp target/honk-assets/wrapper-assets/honc-cold-138.jam assets/honc-cold-138.jam

hoonc-octs-type-138-asset:
    mkdir -p crates/honk/assets target/honk-assets
    target/release/hoonc --dynock-typed --output target/honk-assets/data-import-typed-dynock.jam hoon/probes/hoon-compiler/hoonc_octs_type_probe.hoon hoon
    target/release/extract-hoonc-octs-type target/honk-assets/data-import-typed-dynock.jam crates/honk/assets/hoonc-octs-type-138.jam

build-kernel-assets: build dumb-jam wal-jam miner-jam peek-jam bridge-jam roswell-jam

dumb-jam:
    mkdir -p assets
    target/release/hoonc --output dumb.jam hoon/apps/dumbnet/outer.hoon hoon
    mv dumb.jam assets/dumb.jam

wal-jam:
    mkdir -p assets
    target/release/hoonc --output wal.jam hoon/apps/wallet/wallet.hoon hoon
    mv wal.jam assets/wal.jam

miner-jam:
    mkdir -p assets
    target/release/hoonc --output miner.jam hoon/apps/dumbnet/miner.hoon hoon
    mv miner.jam assets/miner.jam

peek-jam:
    mkdir -p assets
    target/release/hoonc --output peek.jam hoon/apps/peek/peek.hoon hoon
    mv peek.jam assets/peek.jam

bridge-jam:
    mkdir -p assets
    target/release/hoonc --output bridge.jam hoon/apps/bridge/bridge.hoon hoon
    mv bridge.jam assets/bridge.jam

roswell-jam:
    mkdir -p assets
    target/release/hoonc --output roswell.jam hoon/apps/roswell/roswell.hoon hoon
    mv roswell.jam assets/roswell.jam

honk-roswell-kernel:
    mkdir -p assets/native
    cargo run --release -p honk --bin honk -- --new --output assets/native/roswell.jam --prelude hoon/common/hoon.hoon hoon/apps/roswell/roswell.hoon hoon
