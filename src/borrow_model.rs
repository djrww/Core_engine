//! 借用組合模型(借.md 第二部分:3·o²·n² 的解剖與機核判定)。
//!
//! 模型:借用 `b = (π_b, κ_b, o_b)`——place(投影路徑)、種類(shr/mut)、
//! 區間 `[s,e)`。衝突關係(NLL 核心):
//! ```text
//! b1 ⊛ b2 ⟺ π 重疊 ∧ o 重疊 ∧ (κ1,κ2) ≠ (shr,shr)
//! ```
//! κ-矩陣 4 格恰 **3 格衝突**(公式常數 3);每區間 **2 端點**(常數 2);
//! naive 成對檢查 **C(n,2) ~ n²**(常數 n²);區間重疊為二元關係(o²)。
//!
//! 三算法差分:naive O(n²) / sweep-line O(n log n + k) / laminar(嵌套塊)
//! depth×n;外加 MiniDatalog(借.md §2 五謂詞規則原樣)定點求解,
//! `error = ∅ ⟺ ownership 正確`,與 naive 差分對拍;並與既有
//! `rep_dd` 的 red_edges 交叉驗證(同一衝突定義的兩條實作)。

use crate::ast::Interval;
use crate::gen::Rng;
use crate::macro_lab::PrincipleReport;
use std::collections::BTreeSet;

/// 借用種類
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorrowKind {
    Shr,
    Mut,
}

impl BorrowKind {
    pub fn symbol(self) -> &'static str {
        match self {
            BorrowKind::Shr => "shr",
            BorrowKind::Mut => "mut",
        }
    }
}

/// 借用 (π, κ, o)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Borrow {
    /// place:投影路徑(x=[]{}, x.f=[0], x.f.g=[0,1] …)
    pub place: Vec<u32>,
    pub kind: BorrowKind,
    /// 區間 [start, end)(直線碼:CFG 點全序)
    pub region: Interval,
}

/// κ-矩陣中衝突的格子:恰 3 格(公式常數 3 的來源)
pub fn conflict_cells() -> [(BorrowKind, BorrowKind); 3] {
    [
        (BorrowKind::Shr, BorrowKind::Mut),
        (BorrowKind::Mut, BorrowKind::Shr),
        (BorrowKind::Mut, BorrowKind::Mut),
    ]
}

/// 種類衝突:非 (shr,shr) 即衝突
pub fn kinds_conflict(a: BorrowKind, b: BorrowKind) -> bool {
    !(a == BorrowKind::Shr && b == BorrowKind::Shr)
}

/// place 重疊:投影路徑互為前綴(x 與 x.f 重疊;x 與 y 不重疊)
pub fn place_overlaps(p1: &[u32], p2: &[u32]) -> bool {
    p1.len().min(p2.len()) == p1.iter().zip(p2.iter()).take_while(|(a, b)| a == b).count()
}

/// 區間重疊(半開區間端點判定,每區間 2 端點 ⇒ O(1))
pub fn region_overlaps(a: &Interval, b: &Interval) -> bool {
    a.start < b.end && b.start < a.end
}

/// 衝突關係 b1 ⊛ b2
pub fn borrows_conflict(a: &Borrow, b: &Borrow) -> bool {
    place_overlaps(&a.place, &b.place)
        && region_overlaps(&a.region, &b.region)
        && kinds_conflict(a.kind, b.kind)
}

// ---------------------------------------------------------------------------
// naive O(n²):成對檢查,telemetry = 恰 C(n,2)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct NaiveTelemetry {
    pub pairwise_checks: usize,
}

pub fn naive_conflicts(bs: &[Borrow]) -> (Vec<(usize, usize)>, NaiveTelemetry) {
    let mut out = Vec::new();
    let mut tel = NaiveTelemetry::default();
    for i in 0..bs.len() {
        for j in (i + 1)..bs.len() {
            tel.pairwise_checks += 1;
            if borrows_conflict(&bs[i], &bs[j]) {
                out.push((i, j));
            }
        }
    }
    (out, tel)
}

// ---------------------------------------------------------------------------
// sweep-line O(n log n + k):2n 端點事件
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct SweepTelemetry {
    pub endpoint_events: usize,
    pub overlap_probes: usize,
}

pub fn sweep_conflicts(bs: &[Borrow]) -> (Vec<(usize, usize)>, SweepTelemetry) {
    let mut tel = SweepTelemetry::default();
    // 事件 (point, is_end, idx):起點先於同點終點(半開區間:[s,e) 與 [e,f) 不重疊)
    let mut events: Vec<(u32, u8, usize)> = Vec::with_capacity(bs.len() * 2);
    for (i, b) in bs.iter().enumerate() {
        events.push((b.region.start, 0, i));
        events.push((b.region.end, 1, i));
    }
    events.sort();
    tel.endpoint_events = events.len(); // 恰 2n(公式常數 2)
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut active: Vec<usize> = Vec::new();
    for (_, is_end, idx) in events {
        if is_end == 0 {
            for &a in &active {
                tel.overlap_probes += 1;
                let (i, j) = if a < idx { (a, idx) } else { (idx, a) };
                if borrows_conflict(&bs[i], &bs[j]) && seen.insert((i, j)) {
                    out.push((i, j));
                }
            }
            active.push(idx);
        } else {
            active.retain(|&x| x != idx);
        }
    }
    out.sort_unstable();
    (out, tel)
}

// ---------------------------------------------------------------------------
// laminar family({{ }} 嵌套塊引理):不交或包含,衝突只在祖先—後代邊
// ---------------------------------------------------------------------------

/// 區間族是否層狀(任意兩區間:不交 或 包含;無部分重疊)
pub fn laminar_ok(regions: &[Interval]) -> bool {
    for i in 0..regions.len() {
        for j in (i + 1)..regions.len() {
            let (a, b) = (&regions[i], &regions[j]);
            let disjoint = a.end <= b.start || b.end <= a.start;
            let nested =
                (a.start <= b.start && b.end <= a.end) || (b.start <= a.start && a.end <= b.end);
            if !disjoint && !nested {
                return false;
            }
        }
    }
    true
}

fn contains(a: &Interval, b: &Interval) -> bool {
    a.start <= b.start && b.end <= a.end
}

#[derive(Clone, Debug, Default)]
pub struct LaminarTelemetry {
    pub comparisons: usize,
}

/// 層狀族專用:衝突只需檢查棧上祖先(depth × n 而非 C(n,2))
pub fn laminar_conflicts(bs: &[Borrow]) -> (Vec<(usize, usize)>, LaminarTelemetry) {
    let mut tel = LaminarTelemetry::default();
    let mut order: Vec<usize> = (0..bs.len()).collect();
    order.sort_by_key(|&i| (bs[i].region.start, std::cmp::Reverse(bs[i].region.end)));
    let mut stack: Vec<usize> = Vec::new();
    let mut out = Vec::new();
    for &i in &order {
        // 彈出所有不包含 i 的祖先
        while let Some(&top) = stack.last() {
            tel.comparisons += 1;
            if bs[top].region.end < bs[i].region.end {
                stack.pop();
            } else {
                break;
            }
        }
        // 衝突只可能發生在剩餘棧上祖先
        for &anc in &stack {
            tel.comparisons += 1;
            if borrows_conflict(&bs[anc], &bs[i]) {
                let (x, y) = if anc < i { (anc, i) } else { (i, anc) };
                out.push((x, y));
            }
        }
        stack.push(i);
    }
    out.sort_unstable();
    (out, tel)
}

/// 兩區間是否為祖先—後代關係(laminar 下重疊 ⟺ 祖先—後代)
pub fn is_ancestor_descendant(a: &Interval, b: &Interval) -> bool {
    contains(a, b) || contains(b, a)
}

// ---------------------------------------------------------------------------
// MiniDatalog(借.md §2 規則原樣)—— error = ∅ ⟺ ownership 正確
// ---------------------------------------------------------------------------

/// Datalog 項
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DTerm {
    Sym(String),
    Num(u32),
}

/// 事實:(謂詞, 參數列)
pub type DFact = (String, Vec<DTerm>);

/// 規則項:變數(位置索引)或常量
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PTerm {
    Var(u8),
    Con(DTerm),
}

/// 規則:head :- body(合取)
#[derive(Clone, Debug)]
pub struct DRule {
    pub head: (String, Vec<PTerm>),
    pub body: Vec<(String, Vec<PTerm>)>,
}

/// 單調有限域 ⇒ 定點存在唯一;naive 反覆應用至收斂
pub fn datalog_solve(facts: &BTreeSet<DFact>, rules: &[DRule]) -> BTreeSet<DFact> {
    let mut db = facts.clone();
    loop {
        let mut added = Vec::new();
        for r in rules {
            let mut env = Vec::new();
            datalog_match_body(&r.body, 0, &mut env, &db, &r.head, &mut added);
        }
        let before = db.len();
        for f in added {
            db.insert(f);
        }
        if db.len() == before {
            return db;
        }
    }
}

fn datalog_match_body(
    body: &[(String, Vec<PTerm>)],
    at: usize,
    env: &mut Vec<Option<DTerm>>,
    db: &BTreeSet<DFact>,
    head: &(String, Vec<PTerm>),
    out: &mut Vec<DFact>,
) {
    let Some((pred, args)) = body.get(at) else {
        // body 全部滿足:實例化 head
        let head_args: Vec<DTerm> = head
            .1
            .iter()
            .map(|t| match t {
                PTerm::Con(c) => c.clone(),
                PTerm::Var(v) => env
                    .get(*v as usize)
                    .and_then(|o| o.clone())
                    .expect("bound var"),
            })
            .collect();
        out.push((head.0.clone(), head_args));
        return;
    };
    for (p, ts) in db.iter() {
        if p != pred || ts.len() != args.len() {
            continue;
        }
        // 整份環境快照回溯:變數以索引定址,先繫結高索引再繫結低索引時,
        // truncate(長度) 無法還原低索引槽位 —— 必須整份存還原。
        let saved: Vec<Option<DTerm>> = env.clone();
        let mut ok = true;
        for (pat, con) in args.iter().zip(ts.iter()) {
            match pat {
                PTerm::Con(c) if c == con => {}
                PTerm::Var(v) => {
                    let vi = *v as usize;
                    while env.len() <= vi {
                        env.push(None);
                    }
                    match &env[vi] {
                        Some(bound) if bound == con => {}
                        None => env[vi] = Some(con.clone()),
                        Some(_) => {
                            ok = false;
                            break;
                        }
                    }
                }
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            datalog_match_body(body, at + 1, env, db, head, out);
        }
        *env = saved;
    }
}

/// 附件 §2 的三條推導規則(逐字對應)
pub fn polonius_style_rules() -> Vec<DRule> {
    use PTerm::Var;
    vec![
        // borrow_live_at(L, P) :- borrow_region(R, L, _), region_live_at(R, P).
        DRule {
            head: ("borrow_live_at".into(), vec![Var(0), Var(1)]),
            body: vec![
                ("borrow_region".into(), vec![Var(2), Var(0), Var(3)]),
                ("region_live_at".into(), vec![Var(2), Var(1)]),
            ],
        },
        // invalidates(P, L) :- access(P, P2, K2), borrow(L, P1, K1),
        //                     overlaps(P1, P2), conflict_kind(K1, K2).
        DRule {
            head: ("invalidates".into(), vec![Var(0), Var(1)]),
            body: vec![
                ("access".into(), vec![Var(0), Var(2), Var(3)]),
                ("borrow".into(), vec![Var(1), Var(4), Var(5)]),
                ("overlaps".into(), vec![Var(4), Var(2)]),
                ("conflict_kind".into(), vec![Var(5), Var(3)]),
            ],
        },
        // error(P) :- invalidates(P, L), borrow_live_at(L, P).
        DRule {
            head: ("error".into(), vec![Var(0)]),
            body: vec![
                ("invalidates".into(), vec![Var(0), Var(1)]),
                ("borrow_live_at".into(), vec![Var(1), Var(0)]),
            ],
        },
    ]
}

/// 由借用集生成 EDB(直線碼:region [s,e] 存活於 s..=e;簽發點視為一次 access)
pub fn build_edb(bs: &[Borrow]) -> BTreeSet<DFact> {
    use DTerm::{Num, Sym};
    let mut db = BTreeSet::new();
    let place_sym = |p: &[u32]| -> DTerm {
        Sym(format!(
            "p{}",
            p.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("_")
        ))
    };
    for (i, b) in bs.iter().enumerate() {
        let l = Num(i as u32);
        let r = Num(i as u32);
        db.insert((
            "borrow".into(),
            vec![l.clone(), place_sym(&b.place), Sym(b.kind.symbol().into())],
        ));
        // 區間語義:存活於開區間 (s,e)。兩個理由:
        // 1) 簽發點自身的 access 不得觸發自我失效(s ∉ (s,e) ⇒ 同一借用不可能
        //    invalidates 自己 —— 對應 naive 的 i<j 配對不含自身);
        // 2) 起點互異前提下,「s' ∈ (s,e)」與半開重疊「s<e' && s'<e」嚴格等價
        //    (互斥鄰接 [5,7)+[7,9) 也不誤報:7 ∉ (5,7))。
        db.insert((
            "borrow_region".into(),
            vec![r.clone(), l.clone(), Num(b.region.start)],
        ));
        for p in b.region.start + 1..b.region.end {
            db.insert(("region_live_at".into(), vec![r.clone(), Num(p)]));
        }
        db.insert((
            "access".into(),
            vec![
                Num(b.region.start),
                place_sym(&b.place),
                Sym(b.kind.symbol().into()),
            ],
        ));
    }
    // overlaps(π1, π2):place 前綴關係(EDB 事實;Datalog 本身不算前綴,如實申報)
    for i in 0..bs.len() {
        for j in 0..bs.len() {
            if place_overlaps(&bs[i].place, &bs[j].place) {
                db.insert((
                    "overlaps".into(),
                    vec![place_sym(&bs[i].place), place_sym(&bs[j].place)],
                ));
            }
        }
    }
    // conflict_kind:恰 3 格
    for (a, b) in conflict_cells() {
        db.insert((
            "conflict_kind".into(),
            vec![Sym(a.symbol().into()), Sym(b.symbol().into())],
        ));
    }
    db
}

/// Datalog 判定:error 關係(空 ⟺ ownership 正確)
pub fn datalog_errors(bs: &[Borrow]) -> Vec<u32> {
    let db = datalog_solve(&build_edb(bs), &polonius_style_rules());
    let mut errs: Vec<u32> = db
        .iter()
        .filter(|(p, _)| p == "error")
        .filter_map(|(_, args)| match &args[..] {
            [DTerm::Num(p)] => Some(*p),
            _ => None,
        })
        .collect();
    errs.sort_unstable();
    errs.dedup();
    errs
}

// ---------------------------------------------------------------------------
// 搜尋空間會計(3·o²·n² 的逐項對照)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchSpace {
    /// 衝突格數(恆 3)
    pub cells: usize,
    /// 借用對數 C(n,2)
    pub borrow_pairs: usize,
    /// 端點事件數(恆 2n)
    pub endpoint_events: usize,
}

pub fn search_space(n: usize) -> SearchSpace {
    SearchSpace {
        cells: conflict_cells().len(),
        borrow_pairs: n * (n - 1) / 2,
        endpoint_events: 2 * n,
    }
}

impl SearchSpace {
    /// naive 檢查空間的乘積讀法:3 格 × 借用對(每對 O(1) 端點比較)
    pub fn naive_probes(&self) -> usize {
        self.cells * self.borrow_pairs
    }
    pub fn render(&self) -> String {
        format!(
            "cells={} × pairs=C({},2)={} × endpoints={}n ⇒ naive_probes={} (Θ(n²);sweep: {} events + k)",
            self.cells, self.borrow_pairs * 2, self.borrow_pairs, self.endpoint_events, self.naive_probes(), self.endpoint_events
        )
    }
}

// ---------------------------------------------------------------------------
// 與既有 rep_dd 通道的交叉驗證(同一衝突定義的兩條實作)
// ---------------------------------------------------------------------------

/// rep_dd::AState → Borrow 集(storage 為 place 根;事件 id 即索引)
pub fn from_rep_state(state: &crate::rep_dd::AState) -> Vec<Borrow> {
    state
        .evs
        .iter()
        .map(|e| Borrow {
            place: vec![e.storage],
            kind: match e.kind {
                crate::rep_dd::K::Mut => BorrowKind::Mut,
                crate::rep_dd::K::Sh => BorrowKind::Shr,
            },
            region: Interval {
                start: e.it.start,
                end: e.it.end,
            },
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 借用模型門禁(供 bin / verify_all / 測試共用)
// ---------------------------------------------------------------------------

pub fn verify_borrow_model() -> Vec<PrincipleReport> {
    let mut out = Vec::new();

    // ---- B1:κ-矩陣 3 格衝突 ----
    let cells = conflict_cells();
    let b1 = cells.len() == 3
        && !kinds_conflict(BorrowKind::Shr, BorrowKind::Shr)
        && kinds_conflict(BorrowKind::Shr, BorrowKind::Mut)
        && kinds_conflict(BorrowKind::Mut, BorrowKind::Mut);
    out.push(PrincipleReport {
        id: "B1",
        name: "κ-矩陣:4 格恰 3 格衝突(公式常數 3)",
        passed: b1,
        evidence: format!(
            "conflict cells = {} [(shr,mut),(mut,shr),(mut,mut)];(shr,shr) 相容",
            cells.len()
        ),
    });

    // ---- B2/B3:naive = sweep = Datalog 差分(種子確定性)----
    let mut rng = Rng::new(0xB0C0_0001);
    let mut b2_ok = true;
    let mut b3_ok = true;
    let mut tested = 0usize;
    let mut conflicts_seen = 0usize;
    for _ in 0..24 {
        let n = 4 + rng.below(10) as usize;
        let mut bs: Vec<Borrow> = Vec::new();
        let mut used_starts = BTreeSet::new();
        for _ in 0..n {
            // 起點互異(與 rep_dd red_edges 的 b.start > a.start 語義對齊)
            let s = loop {
                let s = rng.below(30) as u32;
                if used_starts.insert(s) {
                    break s;
                }
            };
            let e = s + 1 + rng.below(12) as u32;
            let kind = if rng.below(2) == 0 {
                BorrowKind::Mut
            } else {
                BorrowKind::Shr
            };
            let place = if rng.below(3) == 0 {
                vec![]
            } else if rng.below(2) == 0 {
                vec![0]
            } else {
                vec![1]
            };
            bs.push(Borrow {
                place,
                kind,
                region: Interval { start: s, end: e },
            });
        }
        let (naive, _) = naive_conflicts(&bs);
        let (sw, _) = sweep_conflicts(&bs);
        let dl = datalog_errors(&bs);
        tested += 1;
        conflicts_seen += naive.len();
        if naive != sw {
            b2_ok = false;
        }
        // error 非空 ⟺ naive 衝突非空
        if dl.is_empty() != naive.is_empty() {
            b3_ok = false;
        }
    }
    out.push(PrincipleReport {
        id: "B2",
        name: "naive O(n²) = sweep O(n log n + k)(差分)",
        passed: b2_ok,
        evidence: format!(
            "{} 組隨機借用集(種子 0xB0R0W0001)衝突集全等;共 {} 個衝突對",
            tested, conflicts_seen
        ),
    });
    out.push(PrincipleReport {
        id: "B3",
        name: "Datalog 定點:error=∅ ⟺ naive 無衝突(差分)",
        passed: b3_ok,
        evidence: format!(
            "{} 組隨機借用集,Polonius 風格 3 規則定點判定與 naive 等價",
            tested
        ),
    });

    // ---- B4:層狀族({{ }} 引理)----
    // 由嵌套塊生成層狀借用:深度 4,每層 3 個同層不交區間
    fn gen_laminar(depth: u32, base: u32, len: u32, acc: &mut Vec<Borrow>, rng: &mut Rng) {
        if depth == 0 {
            return;
        }
        for k in 0..3u32 {
            let s = base + k * (len / 3).max(1);
            let e = s + (len / 3).max(1);
            let kind = if rng.below(2) == 0 {
                BorrowKind::Mut
            } else {
                BorrowKind::Shr
            };
            acc.push(Borrow {
                place: vec![],
                kind,
                region: Interval {
                    start: s,
                    end: e.min(base + len),
                },
            });
            gen_laminar(depth - 1, s, (len / 3).max(1), acc, rng);
        }
    }
    let mut rng2 = Rng::new(0x1A11_0002);
    let mut lam = Vec::new();
    gen_laminar(4, 0, 81, &mut lam, &mut rng2);
    let regions: Vec<Interval> = lam.iter().map(|b| b.region).collect();
    let lam_ok = laminar_ok(&regions);
    let (naive_l, nt) = naive_conflicts(&lam);
    let (anc_l, lt) = laminar_conflicts(&lam);
    let all_anc = naive_l
        .iter()
        .all(|&(i, j)| is_ancestor_descendant(&lam[i].region, &lam[j].region));
    let b4 = lam_ok && naive_l == anc_l && all_anc && lt.comparisons < nt.pairwise_checks;
    out.push(PrincipleReport {
        id: "B4",
        name: "層狀族:不交或包含 ⇒ 衝突只在祖先—後代(depth×n)",
        passed: b4,
        evidence: format!(
            "n={} 區間 laminar={};laminar 檢查 {} 次比較 < naive C(n,2)={};兩算法衝突集全等且全為祖先—後代對",
            lam.len(), lam_ok, lt.comparisons, nt.pairwise_checks
        ),
    });

    // ---- B5:搜尋空間會計(3 × C(n,2) × 2n)----
    let n = 12usize;
    let bs: Vec<Borrow> = (0..n)
        .map(|i| Borrow {
            place: vec![],
            kind: if i % 2 == 0 {
                BorrowKind::Mut
            } else {
                BorrowKind::Shr
            },
            region: Interval {
                start: i as u32,
                end: i as u32 + 5,
            },
        })
        .collect();
    let (_, nt5) = naive_conflicts(&bs);
    let (_, st5) = sweep_conflicts(&bs);
    let ss = search_space(n);
    let b5 =
        nt5.pairwise_checks == n * (n - 1) / 2 && st5.endpoint_events == 2 * n && ss.cells == 3;
    out.push(PrincipleReport {
        id: "B5",
        name: "搜尋空間會計:pairwise=C(n,2)、sweep 事件=2n、cells=3",
        passed: b5,
        evidence: format!("n={}: {}", n, ss.render()),
    });

    // ---- B6:與 rep_dd red_edges 交叉驗證 ----
    let mut b6_ok = true;
    let mut b6_n = 0usize;
    for s1 in 0..3u32 {
        for e1 in (s1 + 2)..=5u32 {
            for s2 in (s1 + 1)..4u32 {
                for e2 in (s2 + 1)..=5u32 {
                    let st = crate::testkit::fixtures::two_event_state(s1, e1, s2, e2);
                    let bs = from_rep_state(&st);
                    let (mine, _) = naive_conflicts(&bs);
                    let reds = st.red_edges();
                    // red_edges 以事件 id 對 (a,b) 給出;b6 比較無序對集合
                    let mut red_pairs: BTreeSet<(u32, u32)> = BTreeSet::new();
                    for (a, b) in reds {
                        red_pairs.insert((a.min(b), a.max(b)));
                    }
                    let mine_pairs: BTreeSet<(u32, u32)> =
                        mine.iter().map(|&(i, j)| (i as u32, j as u32)).collect();
                    b6_n += 1;
                    if red_pairs != mine_pairs {
                        // 起點互異由構造保證(s2 > s1);若紅邊語義含儲存/重疊差異即暴露
                        b6_ok = false;
                    }
                }
            }
        }
    }
    out.push(PrincipleReport {
        id: "B6",
        name: "交叉驗證:borrow_model.naive ≡ rep_dd.red_edges(兩通道同一衝突定義)",
        passed: b6_ok,
        evidence: format!("{} 個狀態配置,衝突對集合全等", b6_n),
    });

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflict_matrix_has_exactly_three_cells() {
        assert_eq!(conflict_cells().len(), 3);
        assert!(!kinds_conflict(BorrowKind::Shr, BorrowKind::Shr));
    }

    #[test]
    fn place_prefix_semantics() {
        assert!(place_overlaps(&[], &[0])); // x 與 x.f
        assert!(place_overlaps(&[0, 1], &[0])); // x.f.g 與 x.f
        assert!(!place_overlaps(&[0], &[1])); // x.f 與 x.g
    }

    #[test]
    fn naive_sweep_datalog_agree_on_hand_case() {
        // mut [0,5) 與 shr [2,4) 同 place ⇒ 衝突;與 mut [10,12) 不交
        let bs = vec![
            Borrow {
                place: vec![],
                kind: BorrowKind::Mut,
                region: Interval { start: 0, end: 5 },
            },
            Borrow {
                place: vec![],
                kind: BorrowKind::Shr,
                region: Interval { start: 2, end: 4 },
            },
            Borrow {
                place: vec![],
                kind: BorrowKind::Mut,
                region: Interval { start: 10, end: 12 },
            },
        ];
        let (n, _) = naive_conflicts(&bs);
        let (s, _) = sweep_conflicts(&bs);
        assert_eq!(n, vec![(0, 1)]);
        assert_eq!(n, s);
        let errs = datalog_errors(&bs);
        assert!(!errs.is_empty(), "衝突必須在 Datalog error 中浮現");
        // 相容案例:兩個 shr 重疊
        let ok = vec![
            Borrow {
                place: vec![],
                kind: BorrowKind::Shr,
                region: Interval { start: 0, end: 5 },
            },
            Borrow {
                place: vec![],
                kind: BorrowKind::Shr,
                region: Interval { start: 2, end: 4 },
            },
        ];
        assert!(naive_conflicts(&ok).0.is_empty());
        assert!(datalog_errors(&ok).is_empty());
    }

    #[test]
    fn datalog_boundary_semantics_no_self_or_adjacent_false_positive() {
        // 自我不失效:單一 mut 借用,簽發點 access 不得引發 error
        let solo = vec![Borrow {
            place: vec![0],
            kind: BorrowKind::Mut,
            region: Interval { start: 8, end: 18 },
        }];
        assert!(naive_conflicts(&solo).0.is_empty());
        assert!(
            datalog_errors(&solo).is_empty(),
            "簽發點 access 不得自我失效"
        );

        // 互斥鄰接 [5,7) 與 [7,9):兩者皆無衝突(7 ∉ (5,7))
        let adj = vec![
            Borrow {
                place: vec![],
                kind: BorrowKind::Mut,
                region: Interval { start: 5, end: 7 },
            },
            Borrow {
                place: vec![],
                kind: BorrowKind::Mut,
                region: Interval { start: 7, end: 9 },
            },
        ];
        let (n, _) = naive_conflicts(&adj);
        let dl = datalog_errors(&adj);
        assert!(n.is_empty(), "鄰接半開區間不重疊:{:?}", n);
        assert!(dl.is_empty(), "鄰接不得誤報:{:?}", dl);

        // 包含 [4,9) ⊃ [5,6):兩算法都必須抓到
        let nest = vec![
            Borrow {
                place: vec![1],
                kind: BorrowKind::Mut,
                region: Interval { start: 4, end: 9 },
            },
            Borrow {
                place: vec![1],
                kind: BorrowKind::Mut,
                region: Interval { start: 5, end: 6 },
            },
        ];
        let (n2, _) = naive_conflicts(&nest);
        let dl2 = datalog_errors(&nest);
        assert_eq!(n2, vec![(0, 1)]);
        assert!(!dl2.is_empty());
    }

    #[test]
    fn laminar_family_from_nested_blocks() {
        let regions = vec![
            Interval { start: 0, end: 9 },
            Interval { start: 1, end: 4 },
            Interval { start: 2, end: 3 },
            Interval { start: 5, end: 8 },
        ];
        assert!(laminar_ok(&regions));
        let bad = vec![
            Interval { start: 0, end: 5 },
            Interval { start: 3, end: 8 }, // 部分重疊
        ];
        assert!(!laminar_ok(&bad));
    }

    #[test]
    fn borrow_kind_macro_first_token_discrimination() {
        let (k1, p1) = crate::cl0_borrow_kind!(mut x);
        let (k2, p2) = crate::cl0_borrow_kind!(shr y);
        assert_eq!(k1, BorrowKind::Mut);
        assert_eq!(p1, "x");
        assert_eq!(k2, BorrowKind::Shr);
        assert_eq!(p2, "y");
    }

    #[test]
    fn verify_borrow_model_all_pass() {
        for r in verify_borrow_model() {
            assert!(r.passed, "gate {} failed: {}", r.id, r.evidence);
        }
    }
}
