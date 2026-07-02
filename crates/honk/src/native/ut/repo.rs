use super::*;

thread_local! {
    static REST_DUAL_ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

impl<'a> Ut<'a> {
    /// DIAGNOSTIC: strip BOTH spot forms from a noun — `%dbug` AST wrappers
    /// (`[dbug [spot inner]]`→inner) and `%spot` nock hints
    /// (`[11 [spot ..] body]`→body) — recursively, sharing unchanged subtrees.
    /// Two types that differ ONLY in source spots become equal; a difference that
    /// survives is a genuine structural divergence.
    fn strip_all_spots(slab: &mut NounSlab, space: &NounSpace, n: Noun) -> Noun {
        let Ok(cell) = n.in_space(space).as_cell() else {
            return n;
        };
        let h = cell.head().noun();
        let t = cell.tail().noun();
        if let Ok(tag) = h.in_space(space).as_atom() {
            if tag.eq_bytes(b"dbug") {
                if let Ok(tc) = t.in_space(space).as_cell() {
                    return Self::strip_all_spots(slab, space, tc.tail().noun());
                }
            }
            if tag.as_u64() == Ok(11) {
                if let Ok(tc) = t.in_space(space).as_cell() {
                    let p = tc.head().noun();
                    let q = tc.tail().noun();
                    if let Ok(pc) = p.in_space(space).as_cell() {
                        if let Ok(ptag) = pc.head().noun().in_space(space).as_atom() {
                            if ptag.eq_bytes(b"spot") {
                                return Self::strip_all_spots(slab, space, q);
                            }
                        }
                    }
                }
            }
        }
        let nh = Self::strip_all_spots(slab, space, h);
        let nt = Self::strip_all_spots(slab, space, t);
        if unsafe { nh.as_raw() == h.as_raw() && nt.as_raw() == t.as_raw() } {
            return n;
        }
        T(slab, &[nh, nt])
    }

    /// DIAGNOSTIC: unwrap head-level `%dbug`/`%spot` wrappers (no recursion, no
    /// alloc) — used by the bounded spot-blind compare + preview.
    fn unwrap_spots(mut n: Noun, space: &NounSpace) -> Noun {
        loop {
            let Ok(cell) = n.in_space(space).as_cell() else {
                return n;
            };
            let h = cell.head().noun();
            let t = cell.tail().noun();
            if let Ok(tag) = h.in_space(space).as_atom() {
                if tag.eq_bytes(b"dbug") {
                    if let Ok(tc) = t.in_space(space).as_cell() {
                        n = tc.tail().noun();
                        continue;
                    }
                }
                if tag.as_u64() == Ok(11) {
                    if let Ok(tc) = t.in_space(space).as_cell() {
                        let p = tc.head().noun();
                        if let Ok(pc) = p.in_space(space).as_cell() {
                            if let Ok(ptag) = pc.head().noun().in_space(space).as_atom() {
                                if ptag.eq_bytes(b"spot") {
                                    n = tc.tail().noun();
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
            return n;
        }
    }

    /// DIAGNOSTIC: bounded, zero-alloc structural equality MODULO spots. Big atoms
    /// are treated as equal (benign aura/bits noise); the divergence we hunt is
    /// atom-vs-cell (a resolved `0` vs a `%hold`), which this flags immediately.
    /// `fuel` bounds total work so it can never hang.
    fn spot_blind_eq(a: Noun, b: Noun, space: &NounSpace, fuel: &mut u64) -> bool {
        if *fuel == 0 {
            return true;
        }
        *fuel -= 1;
        let a = Self::unwrap_spots(a, space);
        let b = Self::unwrap_spots(b, space);
        match (a.in_space(space).as_cell(), b.in_space(space).as_cell()) {
            (Ok(ca), Ok(cb)) => {
                Self::spot_blind_eq(ca.head().noun(), cb.head().noun(), space, fuel)
                    && Self::spot_blind_eq(ca.tail().noun(), cb.tail().noun(), space, fuel)
            }
            (Err(_), Err(_)) => {
                match (
                    a.in_space(space).as_atom().ok().and_then(|x| x.as_u64().ok()),
                    b.in_space(space).as_atom().ok().and_then(|x| x.as_u64().ok()),
                ) {
                    (Some(x), Some(y)) => x == y,
                    _ => true,
                }
            }
            _ => false,
        }
    }

    /// DIAGNOSTIC: depth-limited structural preview of a noun (atoms as values or
    /// `%cord`; cells recursed to `depth`, deeper shown as `..`). Spot wrappers
    /// are skipped so the structure reads cleanly.
    fn preview_noun(n: Noun, space: &NounSpace, depth: u32) -> String {
        let n = Self::unwrap_spots(n, space);
        if let Ok(atom) = n.in_space(space).as_atom() {
            if let Ok(v) = atom.as_u64() {
                if (0x20..0x7f_ffff).contains(&v) {
                    if let Ok(s) = atom_to_string(atom) {
                        if s.chars().all(|c| c.is_ascii_graphic()) {
                            return format!("%{s}");
                        }
                    }
                }
                return format!("{v}");
            }
            return "BIG".to_string();
        }
        if depth == 0 {
            return "..".to_string();
        }
        match n.in_space(space).as_cell() {
            Ok(cell) => format!(
                "[{} {}]",
                Self::preview_noun(cell.head().noun(), space, depth - 1),
                Self::preview_noun(cell.tail().noun(), space, depth - 1),
            ),
            Err(_) => "?".to_string(),
        }
    }

    /// DIAGNOSTIC: short tag for a type noun — the head cord for a tagged
    /// `[%tag ..]` type, `atom:<n>` for a small atom (so `0` reads as `atom:0`),
    /// else `cell`.
    fn type_top_tag(t: Noun, space: &NounSpace) -> String {
        if let Ok(atom) = t.in_space(space).as_atom() {
            return match atom.as_u64() {
                Ok(v) => format!("atom:{v}"),
                Err(_) => "atom:big".to_string(),
            };
        }
        if let Ok(cell) = t.in_space(space).as_cell() {
            if let Ok(head) = cell.head().noun().in_space(space).as_atom() {
                if let Ok(s) = atom_to_string(head) {
                    return format!("%{s}");
                }
            }
        }
        "cell".to_string()
    }

    #[cfg(test)]
    fn collect_rest_leg_ids(&mut self, legs: &[(Noun, Noun)]) -> Result<Vec<u64>> {
        let mut unique_leg_ids = Vec::new();
        for (inner, hoon_noun) in legs {
            let leg_id = self.hold_repo_fan_leg_intern_id(*inner, *hoon_noun)?;
            if !unique_leg_ids.contains(&leg_id) {
                unique_leg_ids.push(leg_id);
            }
        }
        Ok(unique_leg_ids)
    }

    fn with_active_rest_leg_ids<R>(
        &mut self,
        leg_ids: &[u64],
        body: impl FnOnce(&mut Self) -> Result<R>,
    ) -> Result<R> {
        if leg_ids.iter().any(|leg_id| {
            self.hold_repo_fan_active_leg_ids
                .binary_search(leg_id)
                .is_ok()
        }) {
            return Err(CompilerError::Noun("rest-loop".to_string()));
        }

        for leg_id in leg_ids.iter().copied() {
            let inserted = self.hold_repo_fan_activate_leg_id(leg_id);
            debug_assert!(
                inserted,
                "rest leg id should only be activated once per scope"
            );
        }

        let result = body(self);

        for leg_id in leg_ids.iter().rev().copied() {
            self.hold_repo_fan_deactivate_leg_id(leg_id);
        }

        result
    }

    #[cfg(test)]
    pub(super) fn with_rest_legs<R>(
        &mut self,
        legs: &[(Noun, Noun)],
        body: impl FnOnce(&mut Self) -> Result<R>,
    ) -> Result<R> {
        if legs.is_empty() {
            return body(self);
        }
        let unique_leg_ids = self.collect_rest_leg_ids(legs)?;
        self.with_active_rest_leg_ids(&unique_leg_ids, body)
    }

    pub(super) fn with_rest_leg_id<R>(
        &mut self,
        leg_id: u64,
        body: impl FnOnce(&mut Self) -> Result<R>,
    ) -> Result<R> {
        self.with_active_rest_leg_ids(&[leg_id], body)
    }

    /// C-final.4: `rest_inner` now takes each leg's inner subject as a NATIVE
    /// type (`NRc<NTy>`) and threads it straight to `play` (which takes a native
    /// subject since C-final.2). This eliminates the previous
    /// `subject -> live_to_noun -> native_of -> play` round-trip on the %hold
    /// resolution path. The fork is still built on the noun path (RT-07
    /// mug-ordering) and lifted to native by the caller (`repo_hold`).
    pub(super) fn rest_inner(&mut self, legs: &[(NRc<NTy>, Noun)]) -> Result<Noun> {
        let mut played = Vec::with_capacity(legs.len());
        for (inner, hoon_noun) in legs {
            let space = self.slab.noun_space();
            let hoon = self.hoon_ast_lookup_result(*hoon_noun).map_err(|err| {
                let tag = Self::hoon_noun_tag(*hoon_noun, &space)
                    .unwrap_or_else(|| "<unknown>".to_string());
                CompilerError::Noun(format!(
                    "native rest: hold ast missing tag={tag} decode_err={err}"
                ))
            })?;
            let play_ty = self.play(inner.clone(), hoon.as_ref())?;
            // DIAGNOSTIC (HONK_PLAY_DUAL): scoped to consensus.hoon ONLY (cheap loc
            // check first, so tx-engine etc. skip the expensive bare-play+strip).
            // Re-play the %dbug-stripped gene and compare the type MODULO ALL SPOTS.
            if std::env::var_os("HONK_PLAY_DUAL").is_some()
                && !REST_DUAL_ACTIVE.with(|c| c.get())
                && self
                    .dbug_locations
                    .last()
                    .and_then(|l| l.file.as_deref())
                    .is_some_and(|f| f.contains("consensus"))
            {
                let space = self.slab.noun_space();
                let bare_noun = Self::strip_dbug_deep(self.slab, &space, *hoon_noun);
                if unsafe { bare_noun.as_raw() != hoon_noun.as_raw() } {
                    REST_DUAL_ACTIVE.with(|c| c.set(true));
                    let bare_play = match self.hoon_ast_lookup_result(bare_noun) {
                        Ok(bh) => self.play(inner.clone(), bh.as_ref()).ok(),
                        Err(_) => None,
                    };
                    REST_DUAL_ACTIVE.with(|c| c.set(false));
                    if let Some(bare_ty) = bare_play {
                        let space = self.slab.noun_space();
                        let a = live_to_noun(&mut self.cx, &play_ty, self.slab);
                        let b = live_to_noun(&mut self.cx, &bare_ty, self.slab);
                        let mut fuel: u64 = 2_000_000;
                        if !Self::spot_blind_eq(a, b, &space, &mut fuel) {
                            let gtag = Self::hoon_noun_tag(*hoon_noun, &space)
                                .unwrap_or_else(|| "?".to_string());
                            eprintln!(
                                "PLAY_REAL_DIVERGE gene_tag={gtag} spotted={} bared={}",
                                Self::type_top_tag(a, &space),
                                Self::type_top_tag(b, &space),
                            );
                            eprintln!("  GENE   ={}", Self::preview_noun(bare_noun, &space, 8));
                            eprintln!("  SPOTTED={}", Self::preview_noun(a, &space, 12));
                            eprintln!("  BARED  ={}", Self::preview_noun(b, &space, 12));
                        }
                    }
                }
            }
            played.push(live_to_noun(&mut self.cx, &play_ty, self.slab));
        }
        self.fork_from_options(played)
    }

    // HOON138:arm=ut:rest lines=10765-10775 map=direct status=partial reviewed=2026-03-06
    #[cfg(test)]
    // HOON138_NOTE:native direct helper for canonical `++rest`; cache policy still wraps this path
    pub(super) fn rest(&mut self, sut: Noun, legs: &[(Noun, Noun)]) -> Result<Noun> {
        let legs_noun = self.rest_legs_noun(legs);
        // C-final.4: rest_inner takes native inner subjects; the test path carries
        // noun legs, so lift each inner to native here (mirrors the old internal
        // native_of in rest_inner exactly).
        let mut native_legs = Vec::with_capacity(legs.len());
        for (inner, hoon_noun) in legs {
            native_legs.push((
                native_of(&mut self.cx, *inner, &self.slab.noun_space())?,
                *hoon_noun,
            ));
        }
        self.with_rest_legs(legs, |ut| {
            if let Some(cached) = ut.rest_boundary_lookup(sut, legs_noun)? {
                return Ok(cached);
            }
            let result = ut.rest_inner(&native_legs)?;
            ut.rest_boundary_store(sut, legs_noun, result)?;
            Ok(result)
        })
    }

    // HOON138:arm=ut:rest lines=10765-10775 map=wrapper status=partial reviewed=2026-03-06
    #[cfg(test)]
    // HOON138_NOTE:scoped native helper for canonical `++rest` fan activation and loop checks
    pub(super) fn with_rest_leg<R>(
        &mut self,
        inner: Noun,
        hoon_noun: Noun,
        body: impl FnOnce(&mut Self) -> Result<R>,
    ) -> Result<R> {
        let leg = [(inner, hoon_noun)];
        self.with_rest_legs(&leg, body)
    }

    fn repo_hold(
        &mut self,
        typ: Noun,
        subject: NRc<NTy>,
        inner: Noun,
        hoon_noun: Noun,
    ) -> Result<NRc<NTy>> {
        // C-final.4: `subject` is the leg's NATIVE inner type (the deepening
        // subject), threaded straight to `play` via `rest_inner`. The noun `inner`
        // (= live_to_noun(subject)) is retained ONLY for the still-noun-keyed
        // leg_id intern + rest_boundary cache (re-key deferred). The noun
        // `legs` for the cache stays (inner_noun, hoon_noun).
        let native_legs = [(subject, hoon_noun)];
        let leg_id = self.hold_repo_fan_leg_id_for_hold_type(typ, inner, hoon_noun)?;
        let legs_noun = self.rest_legs_noun(&[(inner, hoon_noun)]);
        // ATOMIC FLIP (consumer): repo_hold returns native. The rest_boundary
        // cache stays noun-keyed (Phase 1); the noun fork result (RT-07 ordering)
        // is lifted to native at the boundary.
        let result_noun = self.with_rest_leg_id(leg_id, |ut| {
            if let Some(cached) = ut.rest_boundary_lookup(typ, legs_noun)? {
                return Ok(cached);
            }
            let result = ut.rest_inner(&native_legs)?;
            ut.rest_boundary_store(typ, legs_noun, result)?;
            Ok(result)
        })?;
        // repo results are freshly built each recursion level; content-key the
        // decode so structurally-equal expansions reuse one interned `Rc`.
        self.native_of_cached(result_noun)
    }

    pub(super) fn ty_hold_cached(&mut self, inner: Noun, hoon: Noun) -> Result<Noun> {
        let raw_key = (unsafe { inner.as_raw() }, unsafe { hoon.as_raw() });
        if let Some(cached) = self.hold_memo.hold_type_raw.get(&raw_key) {
            return Ok(cached);
        }

        let space = self.slab.noun_space();
        let key = (self.noun_mug_cached(inner), self.noun_mug_cached(hoon));
        if let Some(entries) = self.hold_memo.hold_type.get(&key) {
            let inner_raw = unsafe { inner.as_raw() };
            let hoon_raw = unsafe { hoon.as_raw() };
            for entry in entries.iter().rev() {
                let inner_match = unsafe { entry.inner.raw_equals(&inner) }
                    || unsafe { entry.inner.as_raw() } == inner_raw
                    || noun_eq(entry.inner, inner, &space)?;
                if !inner_match {
                    continue;
                }
                let hoon_match = unsafe { entry.hoon.raw_equals(&hoon) }
                    || unsafe { entry.hoon.as_raw() } == hoon_raw
                    || noun_eq(entry.hoon, hoon, &space)?;
                if hoon_match {
                    self.hold_memo.hold_type_raw.insert_with_limit(
                        raw_key,
                        entry.hold,
                        Self::HOLD_TYPE_CACHE_RAW_KEY_LIMIT,
                    );
                    return Ok(entry.hold);
                }
            }
        }

        let hold = ty_hold(self.slab, inner, hoon);
        self.hold_memo.hold_type_raw.insert_with_limit(
            raw_key,
            hold,
            Self::HOLD_TYPE_CACHE_RAW_KEY_LIMIT,
        );
        let bucket = self
            .hold_memo
            .hold_type
            .ensure_key(key, Self::HOLD_TYPE_CACHE_KEY_LIMIT);
        if bucket.len() >= Self::HOLD_TYPE_CACHE_BUCKET_LIMIT {
            bucket.pop_front();
        }
        bucket.push_back(HoldTypeCacheEntry { inner, hoon, hold });
        Ok(hold)
    }

    // HOON138:arm=ut:repo lines=10754-10763 map=direct status=partial reviewed=2026-03-06
    // HOON138_NOTE:native primary implementation for canonical `++repo`; full parity review is still in progress
    pub(super) fn repo(&mut self, typ: NRc<NTy>) -> Result<NRc<NTy>> {
        // ATOMIC FLIP (consumer, STEP 1): repo reads the native enum directly
        // instead of decoding a type noun. cons_cell mirrors the noun cell_type
        // void-collapse. Hold still routes through the noun rest_inner/play path
        // (gene/subject lowered to noun); the fork rebuild stays on the noun
        // fork_from_options path (RT-07 ordering), lifted back to native.
        match &*typ {
            NTy::Face { inner, .. } => Ok(inner.clone()),
            NTy::Hint { payload, .. } => Ok(payload.clone()),
            NTy::Core { payload, .. } => {
                let head = cons_noun(&mut self.cx);
                Ok(cons_cell(&mut self.cx, head, payload.clone()))
            }
            NTy::Hold { subject, gene } => {
                let subject = subject.clone();
                let gene = gene.clone();
                // `inner`/`typ_noun` lowered ONLY for the still-noun-keyed leg_id +
                // rest_boundary cache key (re-key deferred); the native `subject`
                // threads straight to `play` (C-final.4), dropping the prior
                // subject -> noun -> native round-trip before play.
                let inner = live_to_noun(&mut self.cx, &subject, self.slab);
                let hoon = live_leaf_to_noun(&mut self.cx, &gene, self.slab);
                let typ_noun = live_to_noun(&mut self.cx, &typ, self.slab);
                self.repo_hold(typ_noun, subject, inner, hoon)
            }
            NTy::Noun => {
                let atom = ty_atom(self.slab, "$", None);
                let noun = ty_noun(self.slab);
                let cell = ty_cell(self.slab, noun, noun);
                let fork_noun = self.fork_from_options(vec![atom, cell])?;
                self.native_of_cached(fork_noun)
            }
            _ => Err(CompilerError::Noun("repo-fltt".to_string())),
        }
    }

    /// Noun-bridged `repo` for not-yet-flipped callers (STEP 1): lift the noun
    /// type to native, run native repo, lower the result. Drops as callers flip.
    pub(super) fn repo_noun(&mut self, typ: Noun) -> Result<Noun> {
        // The redo loop calls this per %hold level on freshly-built nouns;
        // content-key the decode so equal subjects reuse one interned `Rc`.
        let native = self.native_of_cached(typ)?;
        let r = self.repo(native)?;
        Ok(live_to_noun(&mut self.cx, &r, self.slab))
    }
}
