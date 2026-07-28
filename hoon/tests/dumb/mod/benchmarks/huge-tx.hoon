/=  helpers  /tests/dumb/helpers
/=  txe  /common/tx-engine
/=  zoon  /common/zoon
/=  dmin  /apps/dumbnet/lib/miner
/=  *  /common/test
/=  *  /common/h-zoon
|%
++  h  ~(. helpers bc-pending-integration-tests:helpers)
++  t  ~(. txe bc-pending-integration-tests:helpers)
++  h-v1  ~(. helpers bc-v1-phase:helpers)
++  t-v1  ~(. txe bc-v1-phase:helpers)
++  bc-legacy-egress
  %*  .  bc-no-timelock:helpers
    ::  Produce hundreds of v0 coinbases, then activate v1 so one current
    ::  transaction can consolidate those legacy notes through %0 spends.
    v1-phase  700
  ==
++  h-egress  ~(. helpers bc-legacy-egress)
++  t-egress  ~(. txe bc-legacy-egress)
++  bench-huge-tx
  =+  [nockchain genesis]=init-nockchain:h
  ::
  ::  add 500 blocks following genesis
  =^  pages  nockchain
    (add-n-pages-integration:h genesis 2.000 nockchain)
  ::
  ::  create huge fan-in transcation with all 500 coinbases
  =/  raw=raw-tx:t
    %-  from-inputs:v0:raw-tx:t
    %-  multi:new:v0:inputs:t
    %+  turn
      (scag 10 pages)
    |=  =page:t
    =/  coin=coinbase:t  (new:v0:coinbase:t page p:default-keys-1:h)
    ?>  ?=(^ -.coin)
    %:  simple-from-note:new:v0:input:t
        p:default-keys-2:h
        coin
        s:default-keys-1:h
    ==
  =^  effects  nockchain
    (~(heard-tx k-by:h nockchain) raw)
  ~
::
::  Incident-shaped benchmark: build one admitted v1 consolidation
::  transaction with hundreds of independent coinbase spends and run the
::  canonical accumulator once at the next candidate height.
++  bench-huge-v1-tx
  =/  con=consensus-state:h-v1  initial-consensus-state:h-v1
  =^  last=page:t-v1  con
    (add-n-pages:h-v1 700 con ~)
  =/  pages=(list page:t-v1)
    %+  turn  ~(val h-by blocks.con)
    |=  pag=local-page:t-v1
    (to-page:local-page:t-v1 pag)
  =/  pages
    %+  scag  674
    %+  skim  pages
    |=  pag=page:t-v1
    ?&  (gte ~(height get:page:t-v1 pag) v1-phase:t-v1)
        (lth ~(height get:page:t-v1 pag) ~(height get:page:t-v1 last))
    ==
  ?>  =(674 (lent pages))
  =/  heavy=block-id:t-v1  (need heaviest-block.con)
  =/  balance=(h-map nname:t-v1 nnote:t-v1)
    (need (~(get h-by balance.con) heavy))
  =/  pks=(list schnorr-pubkey:t-v1)
    ~(tap z-in pubkeys.p:default-keys-1:h-v1)
  =/  m=@  (lent pks)
  =/  [root=hash:t-v1 sc=spend-condition:v1:t-v1 *]
    (make-coinbase-lock:v1:h-v1 m pks)
  =/  pk=schnorr-pubkey:t-v1  (snag 0 pks)
  =/  spends=spends:v1:t-v1
    %+  roll  pages
    |=  [pag=page:t-v1 acc=spends:v1:t-v1]
    =/  coin=nnote:t-v1
      (get-coinbase-from-balance:v1:h-v1 pag balance)
    =/  sed=seed:v1:t-v1
      (make-seed:v1:h-v1 root (sub assets.coin 256) (hash:nnote:t-v1 coin))
    =/  seds=seeds:v1:t-v1
      (~(put z-in *seeds:v1:t-v1) sed)
    =/  sp=spend-1:v1:t-v1
      %*  .  *spend-1:v1:t-v1
        witness  *witness:v1:t-v1
        seeds    seds
        fee      256
      ==
    =/  sig-h=hash:t-v1  (sig-hash:spend-1:v1:t-v1 sp)
    =/  wit=witness:t-v1
      (make-pkh-witness:v1:h-v1 root sc sig-h ~[[s:default-keys-1:h-v1 pk]])
    =.  sp  sp(witness wit)
    (~(put z-by acc) ~(name get:nnote:t-v1 coin) [%1 sp])
  =/  raw=raw-tx:v1:t-v1  (new:raw-tx:v1:t-v1 spends)
  ~&  [%huge-v1-spends ~(wyt z-by spends) %size ~(size get:raw-tx:t-v1 raw)]
  =/  acc=tx-acc:t-v1
    (new:tx-acc:t-v1 `balance +(~(height get:page:t-v1 last)))
  =/  result=(reason:t-v1 tx-acc:t-v1)
    (process:tx-acc:t-v1 acc raw)
  ?>  ?=(%.y -.result)
  ~
::
::  Incident-shaped legacy-wallet benchmark: one current v1 transaction
::  consolidates 674 independent pre-activation v0 coinbase notes through %0
::  spends. This covers the egress path that motivated the liveness repair,
::  including legacy Schnorr verification and grouped v1 output construction.
++  bench-huge-v0-egress
  =/  con=consensus-state:h-egress  initial-consensus-state:h-egress
  =^  last=page:t-egress  con
    (add-n-pages:h-egress 700 con ~)
  ?>  =(700 ~(height get:page:t-egress last))
  =/  pages=(list page:t-egress)
    %+  turn  ~(val h-by blocks.con)
    |=  pag=local-page:t-egress
    (to-page:local-page:t-egress pag)
  =/  pages
    %+  scag  674
    %+  skim  pages
    |=  pag=page:t-egress
    ?&  ?=(^ -.pag)
        !=(0 ~(height get:page:t-egress pag))
        (lth ~(height get:page:t-egress pag) v1-phase:t-egress)
    ==
  ?>  =(674 (lent pages))
  =/  heavy=block-id:t-egress  (need heaviest-block.con)
  =/  balance=(h-map nname:t-egress nnote:t-egress)
    (need (~(get h-by balance.con) heavy))
  =/  pks=(list schnorr-pubkey:t-egress)
    ~(tap z-in pubkeys.p:default-keys-1:h-egress)
  =/  m=@  (lent pks)
  =/  [root=hash:t-egress * *]
    (make-pkh-lock:v1:h-egress m pks)
  =/  fee=coins:t-egress  256
  =/  spends=spends:v1:t-egress
    %+  roll  pages
    |=  [pag=page:t-egress acc=spends:v1:t-egress]
    =/  coin=coinbase:t-egress
      (new:v0:coinbase:t-egress pag p:default-keys-1:h-egress)
    ?>  ?=(^ -.coin)
    ?>  (~(has h-by balance) ~(name get:nnote:t-egress coin))
    =/  sed=seed:v1:t-egress
      (make-seed:v1:h-egress root (sub assets.coin fee) (hash:nnote:t-egress coin))
    =/  seds=seeds:v1:t-egress
      (~(put z-in *seeds:v1:t-egress) sed)
    =/  sp0=spend-0:v1:t-egress
      (new:spend-0:v1:t-egress seds fee)
    =.  sp0
      (sign:spend-0:v1:t-egress sp0 s:default-keys-1:h-egress)
    (~(put z-by acc) ~(name get:nnote:t-egress coin) [%0 sp0])
  =/  raw=raw-tx:v1:t-egress  (new:raw-tx:v1:t-egress spends)
  ~&  [%huge-v0-egress-spends ~(wyt z-by spends) %size ~(size get:raw-tx:t-egress raw)]
  ::  Admit the transaction into the real mining candidate, then prove every
  ::  customer restamp and the final house restore preserve that large body.
  =/  raw-id=tx-id:t-egress  ~(id get:raw-tx:t-egress raw)
  =/  house=mining-state:h-egress  initial-mining-state:h-egress
  =.  house
    (~(heard-new-block dmin house bc-legacy-egress) con *@da)
  =.  house
    (~(heard-new-tx dmin house bc-legacy-egress) raw)
  ?>  (~(has z-in ~(tx-ids get:page:t-egress candidate-block.house)) raw-id)
  =/  customer-shares=(z-map hash:t-egress @)
    =/  pk-hash=hash:t-egress
      (hash:schnorr-pubkey:t-egress default-a-pt-2:h-egress)
    (~(put z-by *(z-map hash:t-egress @)) pk-hash 1)
  =/  customer=mining-state:h-egress
    (~(restamp-candidate dmin house bc-legacy-egress) customer-shares)
  =/  restored=mining-state:h-egress
    (~(restamp-candidate dmin customer bc-legacy-egress) shares.house)
  ?>  !=(candidate-block.house candidate-block.customer)
  ?>  =(~(tx-ids get:page:t-egress candidate-block.house) ~(tx-ids get:page:t-egress candidate-block.customer))
  ?>  =(candidate-acc.house candidate-acc.customer)
  ?>  =(candidate-block.house candidate-block.restored)
  ~
::
::  Frozen reference implementation from before the grouped builder.  This is
::  test-only: it proves the optimized builder emits the exact same noun.
++  legacy-build-v1-outputs
  |=  [raw=raw-tx:v1:t-v1 page-number=page-number:t-v1]
  ^-  outputs:v1:t-v1
  =/  spends-list=(list [nname:t-v1 spend:v1:t-v1])
    ~(tap z-by spends.raw)
  =|  children=(z-map hash:t-v1 output:v1:t-v1)
  |-  ^-  outputs:v1:t-v1
  ?~  spends-list
    %-  ~(gas z-in *(z-set output:v1:t-v1))
    ~(val z-by children)
  =/  sp=spend:v1:t-v1  +.i.spends-list
  =/  sed-list=(list seed:v1:t-v1)
    ?-  -.sp
      %0  ~(tap z-in seeds.+.sp)
      %1  ~(tap z-in seeds.+.sp)
    ==
  =.  children
    %+  roll  sed-list
    |=  [sed=seed:v1:t-v1 acc=_children]
    =/  key=hash:t-v1  lock-root.sed
    =/  mchild=(unit output:v1:t-v1)  (~(get z-by acc) key)
    ?^  mchild
      =*  child  u.mchild
      =/  new-seeds=seeds:v1:t-v1
        (~(put z-in seeds.child) sed)
      =/  new-assets=coins:t-v1
        (add assets.note.child gift.sed)
      =/  normalized-seeds=seeds:v1:t-v1
        %-  ~(gas z-in *seeds:v1:t-v1)
        %+  turn  ~(tap z-in new-seeds)
        |=(s=seed:v1:t-v1 s(output-source ~))
      =/  src-hash=hash:t-v1
        (hash:seeds:v1:t-v1 normalized-seeds)
      =/  src=source:t-v1  [src-hash %.n]
      ?>  ?=(@ -.note.child)
      =/  updated-child=output:v1:t-v1
        :_  new-seeds
        %=  note.child
          assets     new-assets
          name       (new-v1:nname:t-v1 [lock-root.sed src])
          note-data  (~(uni z-by note-data.note.child) note-data.sed)
        ==
      (~(put z-by acc) key updated-child)
    =/  single=seeds:v1:t-v1
      (~(put z-in *seeds:v1:t-v1) sed)
    =/  normalized-single=seeds:v1:t-v1
      (~(put z-in *seeds:v1:t-v1) sed(output-source ~))
    =/  sh=hash:t-v1
      (hash:seeds:v1:t-v1 normalized-single)
    =/  src=source:t-v1  [sh %.n]
    =/  note1=nnote-1:v1:t-v1
      %*  .  *nnote-1:v1:t-v1
        version      %1
        origin-page  page-number
        name         (new-v1:nname:t-v1 [lock-root.sed src])
        note-data    note-data.sed
        assets       gift.sed
      ==
    =/  out=output:v1:t-v1  [note1 single]
    (~(put z-by acc) key out)
  $(spends-list t.spends-list)
::
++  make-builder-seed
  |=  $:  root=hash:t-v1
          gift=coins:t-v1
          parent=hash:t-v1
          src=(unit source:t-v1)
          data=(z-map @tas *)
      ==
  ^-  seed:v1:t-v1
  %*  .  *seed:v1:t-v1
    output-source  src
    lock-root      root
    note-data      data
    gift           gift
    parent-hash    parent
  ==
::
++  make-builder-spend
  |=  seds=seeds:v1:t-v1
  ^-  spend:v1:t-v1
  :-  %1
  %*  .  *spend-1:v1:t-v1
    witness  *witness:v1:t-v1
    seeds    seds
    fee      0
  ==
::
++  make-builder-raw
  |=  sps=spends:v1:t-v1
  ^-  raw-tx:v1:t-v1
  (new:raw-tx:v1:t-v1 sps)
::
++  make-builder-many-spends
  |=  $:  count=@
          output-root=hash:t-v1
          name-root=hash:t-v1
          marker-root=hash:t-v1
      ==
  ^-  spends:v1:t-v1
  =/  empty-data=(z-map @tas *)  *(z-map @tas *)
  =/  i=@  0
  =/  acc=spends:v1:t-v1  *spends:v1:t-v1
  |-
  ?:  =(i count)  acc
  =/  marker=seed:v1:t-v1
    (make-builder-seed marker-root +(i) name-root ~ empty-data)
  =/  parent=hash:t-v1  (hash:seed:v1:t-v1 marker)
  =/  sed=seed:v1:t-v1
    (make-builder-seed output-root 1 parent ~ empty-data)
  =/  seds=seeds:v1:t-v1
    (~(put z-in *seeds:v1:t-v1) sed)
  =/  nam=nname:t-v1
    (new-v1:nname:t-v1 parent [name-root %.n])
  =/  next=spends:v1:t-v1
    (~(put z-by acc) nam (make-builder-spend seds))
  $(i +(i), acc next)
::
++  make-builder-many-seeds
  |=  $:  count=@
          roots=(list hash:t-v1)
          marker-root=hash:t-v1
          parent-root=hash:t-v1
      ==
  ^-  seeds:v1:t-v1
  =/  empty-data=(z-map @tas *)  *(z-map @tas *)
  =/  root-count=@  (lent roots)
  ?>  !=(0 root-count)
  =/  i=@  0
  =/  acc=seeds:v1:t-v1  *seeds:v1:t-v1
  |-
  ?:  =(i count)  acc
  =/  root=hash:t-v1  (snag (mod i root-count) roots)
  =/  marker=seed:v1:t-v1
    (make-builder-seed marker-root +(i) parent-root ~ empty-data)
  =/  parent=hash:t-v1  (hash:seed:v1:t-v1 marker)
  =/  next=seeds:v1:t-v1
    %-  ~(put z-in acc)
    (make-builder-seed root (add 11 i) parent ~ empty-data)
  $(i +(i), acc next)
::
++  grouped-build-v1-outputs
  |=  [raw=raw-tx:v1:t-v1 page-number=page-number:t-v1]
  ^-  outputs:v1:t-v1
  =/  tac=tx:v1:t-v1  (new:tx:v1:t-v1 raw page-number)
  outputs.tac
::
++  test-grouped-v1-outputs-equal-legacy
  =/  empty-data=(z-map @tas *)  *(z-map @tas *)
  ::  Generate three non-zero, correctly aura-typed hashes without relying on
  ::  synthetic @ud literals.
  =/  marker-a=seed:v1:t-v1
    (make-builder-seed *hash:t-v1 1 *hash:t-v1 ~ empty-data)
  =/  marker-b=seed:v1:t-v1
    (make-builder-seed *hash:t-v1 2 *hash:t-v1 ~ empty-data)
  =/  marker-c=seed:v1:t-v1
    (make-builder-seed *hash:t-v1 3 *hash:t-v1 ~ empty-data)
  =/  root-a=hash:t-v1  (hash:seed:v1:t-v1 marker-a)
  =/  root-b=hash:t-v1  (hash:seed:v1:t-v1 marker-b)
  =/  root-c=hash:t-v1  (hash:seed:v1:t-v1 marker-c)
  =/  name-a=nname:t-v1
    (new-v1:nname:t-v1 root-a [root-a %.n])
  =/  name-b=nname:t-v1
    (new-v1:nname:t-v1 root-b [root-b %.n])
  =/  name-c=nname:t-v1
    (new-v1:nname:t-v1 root-c [root-c %.n])
  ::
  ::  Case 1: one ordinary seed.
  =/  seed-one=seed:v1:t-v1
    (make-builder-seed root-a 7 root-b ~ empty-data)
  =/  seeds-one=seeds:v1:t-v1
    (~(put z-in *seeds:v1:t-v1) seed-one)
  =/  spends-one=spends:v1:t-v1
    (~(put z-by *spends:v1:t-v1) name-a (make-builder-spend seeds-one))
  =/  raw-one=raw-tx:v1:t-v1  (make-builder-raw spends-one)
  ::
  ::  Case 2: a large consolidation shape, 674 distinct spends whose seeds
  ::  all feed one output lock root.
  =/  spends-many=spends:v1:t-v1
    (make-builder-many-spends 674 root-a root-b root-c)
  ?>  =(674 ~(wyt z-by spends-many))
  =/  raw-many=raw-tx:v1:t-v1  (make-builder-raw spends-many)
  ::
  ::  Case 3: 48 distinct seeds interleaved across three output lock roots.
  =/  roots=(list hash:t-v1)  ~[root-a root-b root-c]
  =/  seeds-roots=seeds:v1:t-v1
    (make-builder-many-seeds 48 roots root-a root-b)
  ?>  =(48 ~(wyt z-in seeds-roots))
  =/  spends-roots=spends:v1:t-v1
    (~(put z-by *spends:v1:t-v1) name-a (make-builder-spend seeds-roots))
  =/  raw-roots=raw-tx:v1:t-v1  (make-builder-raw spends-roots)
  ::
  ::  Case 4: an explicitly constrained output source.
  =/  seed-source=seed:v1:t-v1
    (make-builder-seed root-b 17 root-c `[root-a %.n] empty-data)
  =/  seeds-source=seeds:v1:t-v1
    (~(put z-in *seeds:v1:t-v1) seed-source)
  =/  spends-source=spends:v1:t-v1
    (~(put z-by *spends:v1:t-v1) name-b (make-builder-spend seeds-source))
  =/  raw-source=raw-tx:v1:t-v1  (make-builder-raw spends-source)
  ::
  ::  Case 5: overlapping and conflicting note-data.  Exact noun equality
  ::  proves that the optimized fold preserves the legacy union direction and
  ::  iteration order.
  =/  data-a=(z-map @tas *)  empty-data
  =.  data-a  (~(put z-by data-a) %shared 101)
  =.  data-a  (~(put z-by data-a) %left 11)
  =/  data-b=(z-map @tas *)  empty-data
  =.  data-b  (~(put z-by data-b) %shared 202)
  =.  data-b  (~(put z-by data-b) %right 22)
  =/  data-c=(z-map @tas *)  empty-data
  =.  data-c  (~(put z-by data-c) %shared 303)
  =.  data-c  (~(put z-by data-c) %left 33)
  =/  seeds-data=seeds:v1:t-v1  *seeds:v1:t-v1
  =.  seeds-data
    %-  ~(put z-in seeds-data)
    (make-builder-seed root-c 19 root-a ~ data-a)
  =.  seeds-data
    %-  ~(put z-in seeds-data)
    (make-builder-seed root-c 23 root-b ~ data-b)
  =.  seeds-data
    %-  ~(put z-in seeds-data)
    (make-builder-seed root-c 29 root-c ~ data-c)
  =/  spends-data=spends:v1:t-v1
    (~(put z-by *spends:v1:t-v1) name-c (make-builder-spend seeds-data))
  =/  raw-data=raw-tx:v1:t-v1  (make-builder-raw spends-data)
  ::
  ::  Case 6: the exact same seed appears in two distinct spends.  The output
  ::  seed set must deduplicate it while its gift is still counted twice.
  =/  seed-duplicate=seed:v1:t-v1
    (make-builder-seed root-b 31 root-a ~ data-a)
  =/  seeds-duplicate=seeds:v1:t-v1
    (~(put z-in *seeds:v1:t-v1) seed-duplicate)
  =/  spends-duplicate=spends:v1:t-v1  *spends:v1:t-v1
  =.  spends-duplicate
    %+  ~(put z-by spends-duplicate)
    name-a
    (make-builder-spend seeds-duplicate)
  =.  spends-duplicate
    %+  ~(put z-by spends-duplicate)
    name-b
    (make-builder-spend seeds-duplicate)
  =/  raw-duplicate=raw-tx:v1:t-v1
    (make-builder-raw spends-duplicate)
  =/  page=page-number:t-v1  4.242
  =/  legacy=(list outputs:v1:t-v1)
    :~  (legacy-build-v1-outputs raw-one page)
        (legacy-build-v1-outputs raw-many page)
        (legacy-build-v1-outputs raw-roots page)
        (legacy-build-v1-outputs raw-source page)
        (legacy-build-v1-outputs raw-data page)
        (legacy-build-v1-outputs raw-duplicate page)
    ==
  =/  grouped=(list outputs:v1:t-v1)
    :~  (grouped-build-v1-outputs raw-one page)
        (grouped-build-v1-outputs raw-many page)
        (grouped-build-v1-outputs raw-roots page)
        (grouped-build-v1-outputs raw-source page)
        (grouped-build-v1-outputs raw-data page)
        (grouped-build-v1-outputs raw-duplicate page)
    ==
  =/  duplicate-output=output:v1:t-v1
    (head ~(tap z-in (snag 5 grouped)))
  =/  expected=[(list outputs:v1:t-v1) coins:t-v1 @]
    [legacy 62 1]
  =/  actual=[(list outputs:v1:t-v1) coins:t-v1 @]
    [grouped assets.note.duplicate-output ~(wyt z-in seeds.duplicate-output)]
  (expect-eq !>(expected) !>(actual))
--
