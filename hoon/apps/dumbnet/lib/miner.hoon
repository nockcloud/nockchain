/=  dk  /apps/dumbnet/lib/types
/=  sp  /common/stark/prover
/=  dumb-transact  /common/tx-engine
/=  dumb-consensus  /apps/dumbnet/lib/consensus
/=  asert  /apps/dumbnet/lib/asert
/=  *  /common/h-zoon
::
:: everything to do with mining and mining state
::
~%  %dumb-miner  ..ut  ~
|_  [m=mining-state:dk =blockchain-constants:dumb-transact]
+*  t  ~(. dumb-transact blockchain-constants)
+|  %admin
::  +set-mining: set .mining
++  set-mining
  ~/  %set-mining
  |=  mine=?
  ^-  mining-state:dk
  m(mining mine)
::  +set-v0-shares: validate and set .v0-shares
++  set-v0-shares
  ~/  %set-v0-shares
  |=  shr=(list [sig:v0:t @])
  =/  s=shares:v0:t  (~(gas z-by *(z-map sig:v0:t @)) shr)
  ?.  (validate:shares:v0:t s)
    ~|('invalid shares' !!)
  m(v0-shares s)
:: set-shares: validate and set .shares
++  set-shares
  ~/  %set-shares
  |=  shr=(list [hash:t @])
  =/  s=shares:t  (~(gas z-by *(z-map hash:t @)) shr)
  ?.  (validate:shares:t s)
    ~|('invalid shares' !!)
  m(shares s)
::
::  Mining requires at least one configured recipient across both reward eras.
++  no-keys-set  ?&(=(*shares:v0:t v0-shares.m) =(*shares:t shares.m))
::
+|  %candidate-block
++  set-pow
  ~/  %set-pow
  |=  prf=proof:sp
  ^-  mining-state:dk
  ?^  -.candidate-block.m  m(pow.candidate-block (some prf))
  m(pow.candidate-block (some prf))
::
++  set-digest
  ^-  mining-state:dk
  ?^  -.candidate-block.m  m(digest.candidate-block (compute-digest:page:t candidate-block.m))
  m(digest.candidate-block (compute-digest:page:t candidate-block.m))
::
++  candidate-block-below-max-size
  %+  lte
    %+  add  (compute-size-without-txs:page:t candidate-block.m)
    (txs-size-by-set:tx-acc:t candidate-acc.m)
  max-block-size:t
::
::  Candidate refill is deliberately limited to one consensus-admissible
::  transaction per refill attempt.  Transaction validation is a non-preemptible
::  Hoon call, so one transaction is the smallest honest unit of work that can
::  be bounded without censoring a large but valid transaction.  Raw size is
::  bounded by the consensus block ceiling; this is not a wall-clock guarantee.
::
::  The libp2p driver emits chain timer pokes every 20 seconds
::  (crates/nockchain/src/config.rs::CHAIN_INTERVAL).  Divide by the cadence
::  before taking the slot modulo so each delivered timer advances the fair
::  selection by one under the normal cadence.  The Rust producer coalesces
::  ticks while a prior timer poke is still in flight, so slow validation cannot
::  queue a train of timer work behind the current transaction.
++  candidate-refill-chain-timer-seconds  20
::
++  candidate-refill-tick
  ~/  %candidate-refill-tick
  |=  now=@da
  ^-  @
  (div (time-in-secs:page:t now) candidate-refill-chain-timer-seconds)
::
::  Select exactly one candidate deterministically from this timer's fair
::  rotation.  h-map traversal is deterministic; advancing the time slot once
::  per refill attempt prevents an invalid/unmineable first transaction from
::  monopolizing every refill attempt without adding persistent kernel state.
::  Building +candidate-txs and materializing its deterministic traversal still
::  scans the retained candidate set once per due refill.  This patch bounds
::  cryptographic transaction processing, not that residual linear metadata
::  cost; an indexed scheduler would require persistent state and migration.
++  candidate-refill-slot
  ~/  %candidate-refill-slot
  |=  [now=@da count=@]
  ^-  @
  ?:  =(0 count)  0
  (mod (candidate-refill-tick now) count)
::
++  pick-refill-tx
  ~/  %pick-refill-tx
  |=  [txs=(list [tx-id=tx-id:t raw=raw-tx:t]) index=@]
  ^-  [tx-id=tx-id:t raw=raw-tx:t]
  ?~  txs
    ~|(%pick-refill-tx-empty !!)
  ?:  =(0 index)
    i.txs
  $(txs t.txs, index (dec index))
::
::  Use transaction payload size alone as a strict lower bound on final block
::  size.  Omitting all header/coinbase overhead makes false rejection
::  impossible while still avoiding full cryptographic validation once the
::  selected payload has already exhausted the entire block budget.
++  candidate-has-room-for-size
  ~/  %candidate-has-room-for-size
  |=  raw-bits=@
  ^-  ?
  (lte (add size.candidate-acc.m raw-bits) max-block-size:t)
::
++  candidate-has-room-for-raw
  ~/  %candidate-has-room-for-raw
  |=  raw=raw-tx:t
  ^-  ?
  (candidate-has-room-for-size ~(size get:raw-tx:t raw))
::
::  A transaction whose input is no longer present in the candidate
::  accumulator is guaranteed to fail +process:tx-acc.  Probe the small input
::  set first so conflicting retained transactions do not repeat signature and
::  witness validation every time the candidate is rebuilt.
++  inputs-in-candidate-balance
  ~/  %inputs-in-candidate-balance
  |=  raw=raw-tx:t
  ^-  ?
  %-  ~(all z-in ~(input-names get:raw-tx:t raw))
  |=  =nname:t
  (~(has h-by balance.candidate-acc.m) nname)
::
::  grab all raw-txs that could possibly be included in block.
::  note that this map could include txs that are not spendable
::  from the current heaviest balance. we rely on the logic inside
::  of process:tx-acc to catch these txs and reject them.
++  candidate-txs
  ~/  %candidate-txs
  |=  c=consensus-state:dk
  ^-  (h-map tx-id:t raw-tx:t)
  |^
    %-  ~(rep h-in candidate-tx-ids)
    |=  [=tx-id:t txs=(h-map tx-id:t raw-tx:t)]
    =/  raw  raw-tx:(~(got h-by raw-txs.c) tx-id)
    ::  Pending forks retain their raw transactions for several blocks.  Once
    ::  one of those transactions has spent an input on the heaviest chain it
    ::  cannot enter our candidate; reject it with the cheap balance-membership
    ::  test instead of repeatedly running the full transaction accumulator.
    ?.  (~(inputs-in-heaviest-balance dumb-consensus c blockchain-constants) raw)
      txs
    (~(put h-by txs) [tx-id raw])
  ::
  ::  union of excluded tx-ids and pending block tx ids
  ::  excluding tx-ids already included in candidate block
  ++  candidate-tx-ids
    %-  %~  dif  h-in
        (~(uni h-in excluded-txs.c) pending-block-tx-ids)
    (zh-silt ~(tx-ids get:page:t candidate-block.m))
  ::
  ::  set of available raw-txs from pending blocks
  ++  pending-block-tx-ids
    ^-  (h-set tx-id:t)
    %-  ~(rep h-by pending-blocks.c)
    |=  [[block-id:t pag=page:t *] all=(h-set tx-id:t)]
    ^-  (h-set tx-id:t)
    %-  ~(rep h-in (zh-silt ~(tx-ids get:page:t pag)))
    |=  [=tx-id:t all=_all]
    ?:  (~(has h-by raw-txs.c) tx-id)
      (~(put h-in all) tx-id)
    all
  --
::
::  +update-candidate-block: boundedly refresh a candidate on a timer poke
::
::  Timestamp refresh remains interval-based (the realnet/default constant is
::  ~m5 / 300 seconds), while transaction refill attempts happen once per
::  delivered timer tick.  The caller invokes this arm only for %timer commands.
::
++  update-candidate-block
  ~/  %update-candidate-block
  |=  [c=consensus-state:dk now=@da]
  ^-  [? mining-state:dk]
  ?:  ?|  =(%.n mining.m)
          =(*page:t candidate-block.m)
          no-keys-set
      ==
    ::  not mining or no candidate block is set so no need to update
    [%.n m]
  =/  timestamp-due=?
    ?:  %+  gte  ~(timestamp get:page:t candidate-block.m)
        (time-in-secs:page:t (sub now update-candidate-interval:t))
      %.n
    %.y
  =?  candidate-block.m  timestamp-due
    ?^  -.candidate-block.m
      candidate-block.m(timestamp (time-in-secs:page:t now))
    candidate-block.m(timestamp (time-in-secs:page:t now))
  =?  m  timestamp-due
    =/  log-message
      %^  cat  3
        'update-candidate-block: Candidate block timestamp updated: '
      (scot %$ ~(timestamp get:page:t candidate-block.m))
    ~>  %slog.[0 log-message]
    m
  =/  [tx-changed=? new-m=mining-state:dk]
    (refill-candidate c now)
  [?|(timestamp-due tx-changed) new-m]
::
++  refill-candidate
  ~/  %refill-candidate
  |=  [c=consensus-state:dk now=@da]
  ^-  [? mining-state:dk]
  ?:  ?|  =(%.n mining.m)
          =(*page:t candidate-block.m)
          no-keys-set
      ==
    [%.n m]
  =/  txs=(list [tx-id=tx-id:t raw=raw-tx:t])
    ~(tap h-by (candidate-txs c))
  ?~  txs  [%.n m]
  =/  slot=@  (candidate-refill-slot now (lent txs))
  =/  [tx-id=tx-id:t raw=raw-tx:t]  (pick-refill-tx txs slot)
  =/  raw-bits=@  ~(size get:raw-tx:t raw)
  ::  These cheap preflights preserve the one-attempt cryptographic budget and
  ::  avoid processing a transaction that is already guaranteed not to fit or
  ::  whose input was consumed by an earlier timer's selected transaction.
  ?.  ?&  (candidate-has-room-for-size raw-bits)
          (inputs-in-candidate-balance raw)
      ==
    [%.n m]
  =/  old-tx-ids=(z-set tx-id:t)
    ~(tx-ids get:page:t candidate-block.m)
  =/  new-m=mining-state:dk  (heard-new-tx-preflighted raw)
  =/  new-tx-ids=(z-set tx-id:t)
    ~(tx-ids get:page:t candidate-block.new-m)
  [!=(old-tx-ids new-tx-ids) new-m]
::
::
::  +heard-new-tx: potentially changes candidate block in reaction to a raw-tx
++  heard-new-tx
  ~/  %heard-new-tx
  |=  raw=raw-tx:t
  ^-  mining-state:dk
  =/  =tx-id:t  ~(id get:raw-tx:t raw)
  =/  log-message
    %+  rap  3
    :~  'heard-new-tx: '
        'Miner received new transaction: '
        (to-b58:hash:t tx-id)
    ==
  ~>  %slog.[0 log-message]
  ::  if the mining pubkey is not set, do nothing
  ?:  ?|(=(%.n mining.m) no-keys-set)  m
  ::
  ::  if the transaction is already in the candidate block, do nothing
  ?:  (~(has z-in ~(tx-ids get:page:t candidate-block.m)) tx-id)
    m
  ::  Full validation is wasted once the payload already selected for this
  ::  candidate plus the raw transaction cannot fit in any legal block.
  =/  raw-bits=@  ~(size get:raw-tx:t raw)
  ?.  (candidate-has-room-for-size raw-bits)
    =/  log-message
        %+  rap  3
        :~  'heard-new-tx: '
            'Transaction '
            (to-b58:hash:t tx-id)
            ' cannot fit in candidate block; skipping validation.'
        ==
    ~>  %slog.[3 log-message]
    m
  ::  Candidate transactions are folded serially.  A prior transaction in the
  ::  fold may already have consumed one of this transaction's inputs even
  ::  though both inputs existed in the heaviest-chain balance used by
  ::  +candidate-txs.  Full processing must reject this transaction, so avoid
  ::  re-running its potentially expensive cryptographic validation.
  ?.  (inputs-in-candidate-balance raw)
    =/  log-message
        %+  rap  3
        :~  'heard-new-tx: '
            'Transaction '
            (to-b58:hash:t tx-id)
            ' cannot be added to candidate block: input absent from candidate balance.'
        ==
    ~>  %slog.[3 log-message]
    m
  (heard-new-tx-preflighted raw)
::
::  Internal transaction admission after deterministic cheap preflights.
::  +heard-new-tx remains the safe public entry point; the timer scheduler uses
::  this arm so the raw transaction is jam-sized only once per refill attempt.
++  heard-new-tx-preflighted
  ~/  %heard-new-tx-preflighted
  |=  raw=raw-tx:t
  ^-  mining-state:dk
  =/  =tx-id:t  ~(id get:raw-tx:t raw)
  :: ::  check to see if block is valid with tx - this checks whether the inputs
  :: ::  exist, whether the new size will exceed block size, and whether timelocks
  :: ::  are valid
  :: =/  tx=(unit tx:t)  (mole |.((new:tx:t raw ~(height get:page:t candidate-block.m))))
  :: ?~  tx
  ::   ::  invalid tx. we don't emit a %liar effect from this because it might
  ::   ::  just not be valid for this particular block
  ::   m
  =.  height.candidate-acc.m  ~(height get:page:t candidate-block.m)
  =/  new-acc=(reason:dk tx-acc:t)
    (process:tx-acc:t candidate-acc.m raw)
  ?.  ?=(%.y -.new-acc)
    =/  log-message
        %+  rap  3
        :~  'heard-new-tx: '
            'Transaction '
            (to-b58:hash:t tx-id)
            ' cannot be added to candidate block.'
        ==
    ~>  %slog.[3 log-message]
    m
  =/  old-mining-state  m
  ::  we can add tx to candidate-block
  =/  new-tx-ids  (~(put z-in ~(tx-ids get:page:t candidate-block.m)) tx-id)
  =.  candidate-block.m
    ?^  -.candidate-block.m
      candidate-block.m(tx-ids new-tx-ids)
    candidate-block.m(tx-ids new-tx-ids)
  =/  old-fees=coins:t  fees.candidate-acc.m
  =.  candidate-acc.m  +.new-acc
  =/  new-fees=coins:t  fees.candidate-acc.m
  =/  log-message-added-tx
      %+  rap  3
      :~  'heard-new-tx: '
          'Added transaction '
          (to-b58:hash:t tx-id)
          ' to the candidate block.'
      ==
  =/  log-message-exceeds-max-size
    %+  rap  3
    :~  'heard-new-tx: '
        'Exceeds max block size, not adding tx: '
        (to-b58:hash:t tx-id)
    ==
  ::  check if new-fees != old-fees to determine if split should be recalculated.
  ::  since we don't have replace-by-fee
  ?:  =(new-fees old-fees)
    ::  fees are equal so no need to recalculate split
    ?.  candidate-block-below-max-size
      ~>  %slog.[3 log-message-exceeds-max-size]
      old-mining-state
    ~>  %slog.[3 log-message-added-tx]
    m
  ::  fees are unequal. for this miner, fees are only ever monotonically
  ::  incremented and so this assertion should never fail.
  ?>  (gth new-fees old-fees)
  =/  fee-diff=coins:t  (sub new-fees old-fees)
  ::  compute old emission+fees
  =/  cb=coinbase-split:t  ~(coinbase get:page:t candidate-block.m)
  =/  old-assets=coins:t
    ?-  -.cb
      %0  %+  roll  ~(val z-by +.cb)
          |=  [c=coins:t sum=coins:t]
          (add c sum)
      %1  %+  roll  ~(val z-by +.cb)
          |=  [c=coins:t sum=coins:t]
          (add c sum)
    ==
  =/  new-assets=coins:t  (add old-assets fee-diff)
  =.  candidate-block.m
    ?^  -.candidate-block.m
      candidate-block.m(coinbase (new:v0:coinbase-split:t new-assets v0-shares.m))
    ::  v1 candidate: dispatch on activation height. Post-activation
    ::  uses the fee-aware 80/20 fund-aware builder (014-aletheia) which
    ::  takes emission and fees separately so the fund slot is computed
    ::  from the subsidy alone; pre-activation retains the existing
    ::  proportional-allocation arm.
    ?:  (pre-asert-activation:t height.candidate-block.m)
      candidate-block.m(coinbase (new:v1:coinbase-split:t new-assets shares.m))
    =/  emission=coins:t
      (emission-calc:coinbase:t height.candidate-block.m)
    candidate-block.m(coinbase (new-with-fund-share:v1:coinbase-split:t emission new-fees shares.m))
  ::  check size of candidate block
  ?.  candidate-block-below-max-size
    ~>  %slog.[3 log-message-exceeds-max-size]
    old-mining-state
  ~>  %slog.[3 log-message-added-tx]
  m
::
::  +heard-new-block: refreshes the candidate block to be mined in reaction to a new block
::
::    when we hear a new heaviest block, we need to update the candidate we're attempting
::    to mine. that means we should update the parent and page number of the block, and carry
::    over any transactions we had previously been attempting to include that werent
::    included in the most recent block.
++  heard-new-block
  ~/  %heard-new-block
  |=  [c=consensus-state:dk now=@da]
  ^-  mining-state:dk
  ?.  mining.m  m
  ::
  ::  do a sanity check that we have a heaviest block, and that the heaviest block
  ::  is not the parent of our current candidate block
  ?~  heaviest-block.c
    ::  genesis block has its own codepath, which is why this conditional does not attempt
    ::  to generate the genesis block
    =/  log-message
      %+  rap  3
      :~  'heard-new-block: '
          'Attempted to generate new candidate block when we have no genesis block'
    ==
  ~>  %slog.[0 log-message]
  m
?:  =(u.heaviest-block.c ~(parent get:page:t candidate-block.m))
    =/  log-message
      %+  rap  3
      :~  'heard-new-block: '
          'Heaviest block unchanged, do not generate new candidate block'
      ==
    ~>  %slog.[0 log-message]
    m
  ?:  no-keys-set
    =/  log-message
      %+  rap  3
      :~  'heard-new-block: '
          'No pubkey(s) set so no new candidate block will be generated'
      ==
    ~>  %slog.[0 log-message]
    m
  =/  log-message
    ^-  @t
    %+  rap  3
    :~  'heard-new-block: '
        'Generating new candidate block with parent: '
        (to-b58:hash:t u.heaviest-block.c)
    ==
  ~>  %slog.[0 log-message]
  =/  parent-local=local-page:t  (~(got h-by blocks.c) u.heaviest-block.c)
  =/  parent=page:t  (to-page:local-page:t parent-local)
  ::  determine the target the candidate (child of .parent) must have.
  =/  candidate-height=@  +(~(height get:page:t parent))
  =/  candidate-target=bignum:bignum:t
    ?:  (post-asert-activation:t candidate-height)
      (~(compute-target-asert dumb-consensus c blockchain-constants) %zk candidate-height u.heaviest-block.c)
    (~(got h-by targets.c) u.heaviest-block.c)
  =.  candidate-block.m
    ?^  -.parent
      ::  v0 parent -
      ::    if candidate height is less than cutoff, use v0 new-candidate with v0 shares
      ::    otherwise use v1 new-candidate with v1 shares
      ?:  (lth +(height.parent) v1-phase.blockchain-constants)
        (new-candidate:v0:page:t parent now candidate-target v0-shares.m)
      (new-candidate:page:t parent now candidate-target shares.m asert-phase.blockchain-constants)
    ::  v1 parent - use v1 new-candidate with v1 shares
    (new-candidate:page:t parent now candidate-target shares.m asert-phase.blockchain-constants)
  =.  candidate-acc.m
    %+  new:tx-acc:t
      (~(get h-by balance.c) u.heaviest-block.c)
    ~(height get:page:t candidate-block.m)
  ::
  ::  Publish the fresh empty candidate immediately.  Retained transactions are
  ::  refilled later, one deterministic attempt per timer poke, so accepting a
  ::  new heaviest block never synchronously folds the entire mempool.
  m
--
