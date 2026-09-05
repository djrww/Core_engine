//! fuzz_engine —— 屬性測試套件引擎與定律檢查核心(DL-008 下沉第 2 批)。
//!
//! 過去 `bin/fuzz.rs`(423 行)與 `bin/cl0r0.rs`(265 行)各自內聯:
//! 九律屬性檢查迴圈、L7b 迭代淨化、樹公理檢查與修剪計劃演示 ——
//! bin 層無法單測(llvm-cov 量測空洞)。本模組把它們抽為可單測的單一真相:
//! * [`FuzzConfig`]/[`run_property_suite`] —— 種子確定性的整套屬性套件
//!   (L1/L2/L5/L6/L7a/L7b/M2/M4/M5/REUSE/L8/L9 + 窮舉小樣);
//! * [`iterative_purify`]/[`residual_bad_errors`] —— L7b 迭代淨化(fuzz 與
//!   cl0r0 演示共用同一份實作);
//! * [`tree_axiom_checks`] —— 樹公理六項檢查(cl0r0 演示);
//! * [`normalize_plan`] —— 事實層 → AState → 規範修剪計劃(cl0r0 演示)。
//!
//! bin 只剩:配置讀取(env)、報告渲染、退出碼。

use crate::ast::{self, EvKind, Track};
use crate::edit;
use crate::gen::{gen_edit, gen_garbage, gen_half_file, gen_legal, Rng};
use crate::parse::{euler_characteristic, parse, reparse, Kind};
use crate::rep::{self, AState, Ev, Menu, Policy, K};

// ===========================================================================
// L7b 迭代淨化(fuzz 套件與 cl0r0 演示的單一真相)
// ===========================================================================

/// 迭代淨化結果:反覆移除極大錯誤跨度至不動點(≤ 8 輪)。
pub struct PurifyOutcome {
    pub purified: String,
    pub rounds: usize,
    pub seams: Vec<u32>,
}

/// 反覆挖除極大錯誤跨度,直到無錯誤跨度或收斂(上限 8 輪)。
pub fn iterative_purify(src: &str) -> PurifyOutcome {
    let mut cur = src.to_string();
    let mut seams: Vec<u32> = Vec::new();
    let mut rounds = 0usize;
    while let Ok(rec) = parse(&cur) {
        let spans = rec.maximal_error_spans();
        if spans.is_empty() {
            break;
        }
        let mut next = String::new();
        let mut last = 0u32;
        for sp in &spans {
            next.push_str(&cur[last as usize..sp.start as usize]);
            seams.push(sp.start);
            seams.push(sp.end);
            last = sp.end;
        }
        next.push_str(&cur[last as usize..]);
        if next == cur {
            break;
        }
        cur = next;
        rounds += 1;
        if rounds >= 8 {
            break;
        }
    }
    PurifyOutcome {
        purified: cur,
        rounds,
        seams,
    }
}

/// 淨化殘餘檢查:錯誤節點只允許「空跨度 ∧ 位於切縫/EOF」(缺失內容語義)。
pub fn residual_bad_errors(cur: &str, seams: &[u32]) -> usize {
    let fin = match parse(cur) {
        Ok(t) => t,
        Err(_) => return usize::MAX,
    };
    let mut bad = 0usize;
    for n in &fin.nodes {
        if n.kind != Kind::Error {
            continue;
        }
        let at_seam = n.span.start as usize == cur.len() || seams.contains(&n.span.start);
        if !n.span.is_empty() || !at_seam {
            bad += 1;
        }
    }
    bad
}

// ===========================================================================
// 樹公理檢查(cl0r0 演示)
// ===========================================================================

/// 六項樹公理檢查結果(L1/L2/L5/連續性/樹形/χ=1)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeAxiomChecks {
    pub roundtrip: bool,
    pub determinism: bool,
    pub laminar: bool,
    pub continuity: bool,
    pub tree_shapes: bool,
    pub euler_is_one: bool,
}

impl TreeAxiomChecks {
    pub fn all_ok(&self) -> bool {
        self.roundtrip
            && self.determinism
            && self.laminar
            && self.continuity
            && self.tree_shapes
            && self.euler_is_one
    }
}

/// 對單一源碼執行六項樹公理檢查(全化解析:任何輸入都產樹)。
pub fn tree_axiom_checks(src: &str) -> TreeAxiomChecks {
    let t = match parse(src) {
        Ok(t) => t,
        Err(_) => {
            return TreeAxiomChecks {
                roundtrip: false,
                determinism: false,
                laminar: false,
                continuity: false,
                tree_shapes: false,
                euler_is_one: false,
            }
        }
    };
    let determinism = parse(src).map(|t2| t2.sexp() == t.sexp()).unwrap_or(false);
    TreeAxiomChecks {
        roundtrip: t.unparse() == src,
        determinism,
        laminar: t.laminar_ok(),
        continuity: t.validate_continuity().is_ok(),
        tree_shapes: t.validate_tree_shapes().is_ok(),
        euler_is_one: euler_characteristic(&t) == 1,
    }
}

// ===========================================================================
// 修剪計劃(cl0r0 幾何演示:事實層 → AState → 規範修剪)
// ===========================================================================

/// 規範修剪計劃:每步 (規則標籤, 修剪前紅邊, 修剪後紅邊)。
pub struct NormalizePlanReport {
    pub initial_red: usize,
    pub steps: usize,
    pub final_red: usize,
    pub plan: Vec<(String, usize, usize)>,
}

/// 把事實層投影為 AState(referent 軌),運行規範修剪菜單至正規形。
pub fn normalize_plan(src: &str) -> NormalizePlanReport {
    let t = match parse(src) {
        Ok(t) => t,
        Err(_) => {
            return NormalizePlanReport {
                initial_red: 0,
                steps: 0,
                final_red: 0,
                plan: Vec::new(),
            }
        }
    };
    let facts = ast::extract(&t);
    let (ivs, events) = ast::intervals(&facts, Track::Referent);
    let mut evs: Vec<Ev> = Vec::new();
    // 潛伏 bug 修復(DL-008):舊演示把全部事件投影到 storage 0(所有變數
    // 互相別名)且以全域事件索引取綁定區間 —— 跨綁定同起點衝突修剪不掉,
    // 舊 bin 靜默印 ✗。正確語義:storage = 綁定(同名記憶體才衝突),
    // 區間 = 該綁定區間表中按綁定內事件序數取值。
    let mut ordinal = vec![0usize; facts.bindings.len()];
    for (i, ev) in events.iter().enumerate() {
        let kind = match ev.kind {
            EvKind::BorrowMut | EvKind::Move | EvKind::Deref => K::Mut,
            _ => K::Sh,
        };
        let ord = ordinal[ev.binding];
        ordinal[ev.binding] += 1;
        evs.push(Ev {
            id: i as u32,
            storage: ev.binding as u32,
            kind,
            it: ivs[ev.binding].get(ord).copied().unwrap_or(ast::Interval {
                start: ev.span.start,
                end: ev.span.end,
            }),
        });
    }
    let s0 = AState::new(evs);
    let initial_red = s0.red_edges().len();
    let (nf, steps) = rep::normalize(s0.clone(), Menu::CommutativeTrim, Policy::Guarded);
    let mut st = s0;
    let mut plan = Vec::new();
    while let Some((s2, r)) = rep::step(&st, Menu::CommutativeTrim, Policy::Guarded) {
        plan.push((r.label(), st.red_edges().len(), s2.red_edges().len()));
        st = s2;
    }
    NormalizePlanReport {
        initial_red,
        steps,
        final_red: nf.red_edges().len(),
        plan,
    }
}

// ===========================================================================
// 三軌衝突圖摘要(cl0r0 演示:紅邊格式化 + T2 χ=ω 計算)
// ===========================================================================

/// 單軌摘要:紅邊(已格式化)+ 區間圖著色定理 T2 實例。
pub struct TrackSummary {
    pub label: &'static str,
    pub edge_lines: Vec<String>,
    pub omega: usize,
    pub chi: usize,
}

/// 對三軌(lexical/nll/referent)各自計算紅邊與 ω/χ(χ=ω ⇒ 完美圖)。
pub fn track_summaries(facts: &ast::Facts) -> Vec<TrackSummary> {
    let mut out = Vec::new();
    for track in [Track::Lexical, Track::Nll, Track::Referent] {
        let edges = ast::red_edges(facts, track);
        let edge_lines = edges
            .iter()
            .map(|e| {
                format!(
                    "({}, {}) on {} @{} [{},{})",
                    facts.events[e.a].kind.label(),
                    facts.events[e.b].kind.label(),
                    facts.bindings[e.binding].name,
                    e.span,
                    facts.events[e.a].span.start,
                    facts.events[e.b].span.start
                )
            })
            .collect();
        let (iv, _) = ast::intervals(facts, track);
        let all: Vec<ast::Interval> = iv.iter().flatten().copied().collect();
        out.push(TrackSummary {
            label: track.label(),
            edge_lines,
            omega: ast::max_clique(&all),
            chi: ast::greedy_chromatic(&all),
        });
    }
    out
}

/// 事實層摘要行(綁定/事件/借鏡,已格式化;cl0r0 演示用)。
pub fn facts_summary_lines(facts: &ast::Facts) -> Vec<String> {
    let mut out = Vec::new();
    for (i, b) in facts.bindings.iter().enumerate() {
        out.push(format!(
            "binding[{}] {} @{} (mut={} param={})",
            i, b.name, b.span, b.mutable, b.is_param
        ));
    }
    for e in &facts.events {
        out.push(format!(
            "event: {} {} @{}",
            e.kind.label(),
            facts.bindings[e.binding].name,
            e.span
        ));
    }
    for l in &facts.links {
        out.push(format!(
            "borrow-link: {} = {} {} @{}",
            facts.bindings[l.ref_binding].name,
            l.kind.label(),
            facts.bindings[l.src_binding].name,
            l.span
        ));
    }
    out
}

// ===========================================================================
// 屬性套件引擎(bin/fuzz 的核心)
// ===========================================================================

/// 套件配置(env 驅動;`minimal()` 供單測快速跑通)。
#[derive(Clone, Copy, Debug)]
pub struct FuzzConfig {
    pub main_iter: usize,
    pub legal_iter: usize,
    pub half_iter: usize,
    pub edit_iter: usize,
    pub reparse_iter: usize,
    pub l8_iter: usize,
}

impl Default for FuzzConfig {
    fn default() -> Self {
        FuzzConfig {
            main_iter: 4000,
            legal_iter: 800,
            half_iter: 1500,
            edit_iter: 500,
            reparse_iter: 200,
            l8_iter: 300,
        }
    }
}

impl FuzzConfig {
    /// 單測用迷你配置(每段少量迭代,秒級完成)。
    pub fn minimal() -> Self {
        FuzzConfig {
            main_iter: 24,
            legal_iter: 8,
            half_iter: 8,
            edit_iter: 8,
            reparse_iter: 4,
            l8_iter: 8,
        }
    }

    /// 環境變量驅動(解析失敗或未設 ⇒ 回退預設;F-08 單一真相)。
    pub fn from_env() -> Self {
        let iters = |name: &str, default: usize| {
            std::env::var(name)
                .ok()
                .and_then(|v| v.parse().ok())
                .filter(|&n| n > 0)
                .unwrap_or(default)
        };
        FuzzConfig {
            main_iter: iters("FUZZ_ITERATIONS", Self::default().main_iter),
            legal_iter: iters("FUZZ_LEGAL_ITERATIONS", Self::default().legal_iter),
            half_iter: iters("FUZZ_HALF_ITERATIONS", Self::default().half_iter),
            edit_iter: iters("FUZZ_EDIT_ITERATIONS", Self::default().edit_iter),
            reparse_iter: iters("FUZZ_REPARSE_ITERATIONS", Self::default().reparse_iter),
            l8_iter: iters("FUZZ_L8_ITERATIONS", Self::default().l8_iter),
        }
    }
}

/// 單律統計
#[derive(Clone, Debug)]
pub struct LawStat {
    pub law: String,
    pub checked: usize,
    pub failed: usize,
}

/// 套件報告:逐律統計 + 失敗明細(調用方負責渲染)。
#[derive(Clone, Debug, Default)]
pub struct FuzzReport {
    pub stats: Vec<LawStat>,
    pub failures: Vec<String>,
}

impl FuzzReport {
    fn record(&mut self, law: &str, ok: bool, detail: String) {
        if let Some(e) = self.stats.iter_mut().find(|s| s.law == law) {
            e.checked += 1;
            if !ok {
                e.failed += 1;
            }
        } else {
            self.stats.push(LawStat {
                law: law.to_string(),
                checked: 1,
                failed: usize::from(!ok),
            });
        }
        if !ok {
            self.failures.push(format!("[{}] {}", law, detail));
        }
    }

    pub fn total_fail(&self) -> usize {
        self.stats.iter().map(|s| s.failed).sum()
    }
}

/// 窮舉小樣 alphabet^≤depth(深度上限 4;回呼逐字串)。
fn exhaust(alphabet: &[&str], depth: usize, max: usize, cur: &mut String, f: &mut dyn FnMut(&str)) {
    if depth >= max {
        f(cur);
        return;
    }
    for a in alphabet {
        cur.push_str(a);
        exhaust(alphabet, depth + 1, max, cur, f);
        cur.truncate(cur.len() - a.len());
    }
}

/// 種子確定性的整套屬性套件(失敗記入報告,不打印、不退出)。
pub fn run_property_suite(seed: u64, cfg: &FuzzConfig) -> FuzzReport {
    let mut rng = Rng::new(seed);
    let mut rep = FuzzReport::default();

    // ---------- L1/L2/L5/連續性/樹公理:任意輸入(含髒);L7a 合法 ⇒ 無 ERROR ----------
    for i in 0..cfg.main_iter {
        let src = if i % 3 == 0 {
            gen_legal(&mut rng)
        } else {
            gen_garbage(&mut rng, 30)
        };
        let t = match parse(&src) {
            Ok(t) => t,
            Err(e) => {
                rep.record("L1", false, format!("parse err {:?}", e));
                continue;
            }
        };
        rep.record("L1", t.unparse() == src, format!("roundtrip {:?}", src));
        let t2 = match parse(&src) {
            Ok(t) => t,
            Err(_) => {
                rep.record("L2", false, "second parse err".into());
                continue;
            }
        };
        rep.record("L2", t2.sexp() == t.sexp(), "determinism".into());
        rep.record("L5", t.laminar_ok(), format!("laminar {}", src));
        rep.record("L1", t.validate_continuity().is_ok(), "continuity".into());
        rep.record("L1", t.validate_tree_shapes().is_ok(), "treeaxiom".into());
        if i % 3 == 0 {
            rep.record(
                "L7a",
                !t.has_error(),
                format!("legal src has errors: {}", src),
            );
        }
    }

    // ---------- 合法程式密集樣本:L7a + L1 + L6 投影一致 ----------
    for _ in 0..cfg.legal_iter {
        let src = gen_legal(&mut rng);
        let t = match parse(&src) {
            Ok(t) => t,
            Err(_) => {
                rep.record("L7a", false, "legal parse err".into());
                continue;
            }
        };
        rep.record("L7a", !t.has_error(), format!("false error: {}", src));
        rep.record("L1", t.unparse() == src, format!("legal-roundtrip {}", src));
        let named1 = t.named_sexp();
        let consistent = parse(&src)
            .map(|t2| t2.named_sexp() == named1)
            .unwrap_or(false);
        rep.record("L6", consistent, "projection-consistency".into());
    }

    // ---------- L7b:寫一半檔案(極大跨度互不嵌套 + 迭代淨化) ----------
    for _ in 0..cfg.half_iter {
        let legal = gen_legal(&mut rng);
        let half = gen_half_file(&mut rng, &legal);
        let t = match parse(&half) {
            Ok(t) => t,
            Err(_) => {
                rep.record("L7b", false, "half parse err".into());
                continue;
            }
        };
        let spans = t.maximal_error_spans();
        let mut nested_ok = true;
        for a in 0..spans.len() {
            for b in (a + 1)..spans.len() {
                let strict = (spans[a].start < spans[b].start && spans[b].end < spans[a].end)
                    || (spans[b].start < spans[a].start && spans[a].end < spans[b].end);
                if strict {
                    nested_ok = false;
                }
            }
        }
        rep.record("L7b", nested_ok, format!("non-nested {:?}", spans));
        let out = iterative_purify(&half);
        let bad = residual_bad_errors(&out.purified, &out.seams);
        let ok = bad == 0 && out.rounds < 8;
        rep.record(
            "L7b",
            ok,
            format!("purify half={:?} bad={} rounds={}", half, bad, out.rounds),
        );
    }

    // ---------- 窮舉小樣(L1/L5) ----------
    let alphabet = [
        "x", "1", ";", "{", "}", "(", ")", "&", "=", "let", "fn", "if", " ",
    ];
    exhaust(&alphabet, 0, 4, &mut String::new(), &mut |s| {
        if let Ok(t) = parse(s) {
            rep.record("L1", t.unparse() == s, format!("exhaustive {:?}", s));
            rep.record("L5", t.laminar_ok(), format!("exhaustive {:?}", s));
        }
    });

    // ---------- 編輯單體:M4 複合 / M5 互不重疊歸併 / M2 位移和 ----------
    for _ in 0..cfg.edit_iter {
        let src = gen_legal(&mut rng);
        let e1 = gen_edit(&mut rng, src.len());
        let s1 = edit::apply(&src, &e1);
        let e2 = gen_edit(&mut rng, s1.len());
        let s2 = edit::apply(&s1, &e2);
        if let Some(combo) = edit::compose(&e1, &e2) {
            let s3 = edit::apply_all(&src, &combo);
            rep.record(
                "M4",
                s2 == s3,
                format!("apply-compose {:?} / {:?}", src, s2),
            );
            let f2 = gen_edit(&mut rng, src.len());
            if edit::is_pairwise_disjoint(&[e1.clone(), f2.clone()]) {
                let a = edit::apply_all(&src, &[e1.clone(), f2.clone()]);
                let b = edit::apply_all(&src, &[f2.clone(), e1.clone()]);
                rep.record("M5", a == b, "order-independence".into());
            }
        }
        let p = (e1.old_end + e2.old_end + 2).min(src.len() as u32);
        if let Some(x) = e1.shift(p) {
            if let Some(y) = e2.shift(x) {
                let total = e1.delta() + e2.delta();
                let ok = !(p > e1.old_end && x > e2.old_end && y as i64 != p as i64 + total);
                rep.record("M2", ok, format!("p={} y={} total={}", p, y, total));
            }
        }
    }

    // ---------- 增量層工具(重用率統計,非律級) ----------
    let mut reuse_seen = false;
    for _ in 0..cfg.reparse_iter {
        let src = gen_legal(&mut rng);
        let t = match parse(&src) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let e = gen_edit(&mut rng, src.len());
        let new_src = edit::apply(&src, &e);
        if let Ok(out) = reparse(&t, &new_src, std::slice::from_ref(&e)) {
            reuse_seen |= out.reused > 0;
        }
    }
    rep.stats.push(LawStat {
        law: "REUSE".to_string(),
        checked: 1,
        failed: usize::from(!reuse_seen),
    });

    // ---------- L8/L9 抽查 ----------
    for _ in 0..cfg.l8_iter {
        let n = 2 + rng.below(3) as usize;
        let mut evs = Vec::new();
        let mut p = 0u32;
        for i in 0..n {
            let len = 1 + rng.below(4) as u32;
            let kind = if rng.chance(1, 2) { K::Mut } else { K::Sh };
            evs.push(Ev {
                id: i as u32,
                storage: 0,
                kind,
                it: ast::Interval {
                    start: p,
                    end: p + len,
                },
            });
            p += len;
        }
        let s = AState::new(evs);
        let l8_ok = rep::l8_check(&s, Menu::CommutativeTrim, Policy::Guarded).is_none();
        rep.record("L8", l8_ok, "measure not decreasing".into());
        let (nf, steps) = rep::normalize(s, Menu::CommutativeTrim, Policy::Guarded);
        rep.record(
            "L9",
            nf.red_edges().is_empty() && steps <= 100,
            format!("normalize steps={} red={}", steps, nf.red_edges().len()),
        );
    }

    rep
}

// ===========================================================================
// 測試(DL-008:下沉後可單測)
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;

    const DEMO: &str = "fn main() {\n let mut x = 1;\n let r = &mut x;\n let y = x + 1;\n let z = *r;\n while z < 10 { f(y); }\n if z == 3 { f(x); } else { g(); }\n }";

    #[test]
    fn minimal_property_suite_has_zero_failures() {
        let rep = run_property_suite(0xC10_2024_0001, &FuzzConfig::minimal());
        assert_eq!(rep.total_fail(), 0, "失敗明細:{:?}", rep.failures);
        // 套件覆蓋的律全部有檢查紀錄
        for law in [
            "L1", "L2", "L5", "L6", "L7a", "L7b", "M2", "M4", "L8", "L9", "REUSE",
        ] {
            assert!(
                rep.stats.iter().any(|s| s.law == law),
                "缺 {} 統計(得 {:?})",
                law,
                rep.stats.iter().map(|s| s.law.clone()).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn suite_is_seed_deterministic() {
        let a = run_property_suite(0xD1_0002, &FuzzConfig::minimal());
        let b = run_property_suite(0xD1_0002, &FuzzConfig::minimal());
        assert_eq!(a.stats.len(), b.stats.len());
        for (x, y) in a.stats.iter().zip(b.stats.iter()) {
            assert_eq!(
                (x.law.clone(), x.checked, x.failed),
                (y.law.clone(), y.checked, y.failed)
            );
        }
    }

    #[test]
    fn tree_axioms_hold_on_legal_source_and_report_parse_failure() {
        let c = tree_axiom_checks(DEMO);
        assert!(c.all_ok(), "{:?}", c);
        // 全化解析:髒輸入也應產樹並通過公理(而非解析失敗)
        let dirty = tree_axiom_checks("@@@ )) let x =");
        assert!(dirty.all_ok(), "全化:髒輸入仍滿足樹公理 {:?}", dirty);
    }

    #[test]
    fn purify_leaves_only_seam_or_eof_empty_errors() {
        for half in [
            "fn main() {\n  let mut x = 1;\n  let r = &mut",
            "fn main() {\n  let y = x +",
            "fn main() { if x { } else {",
            "@@@",
        ] {
            let out = iterative_purify(half);
            let bad = residual_bad_errors(&out.purified, &out.seams);
            assert_eq!(bad, 0, "{:?} 殘餘 {} 個非法錯誤", half, bad);
            assert!(out.rounds < 8, "{:?} 淨化未收斂", half);
        }
    }

    #[test]
    fn normalize_plan_clears_red_edges_on_conflict_sample() {
        let r = normalize_plan(DEMO);
        assert!(r.initial_red > 0, "演示樣本應含紅邊(&mut x 期間讀 x)");
        assert_eq!(r.final_red, 0, "規範修剪 ⇒ 紅邊清零");
        assert_eq!(r.plan.len(), r.steps);
        for (label, before, after) in &r.plan {
            assert!(!label.is_empty());
            assert!(
                after <= before,
                "每步紅邊不增:{} {}→{}",
                label,
                before,
                after
            );
        }
    }

    #[test]
    fn facts_summary_lines_cover_three_kinds() {
        let t = parse(DEMO).unwrap();
        let facts = ast::extract(&t);
        let lines = facts_summary_lines(&facts);
        assert!(lines
            .iter()
            .any(|l| l.starts_with("binding[") && l.contains(" x ")));
        assert!(lines
            .iter()
            .any(|l| l.starts_with("event: ") && l.contains("&mut x")));
        assert!(lines
            .iter()
            .any(|l| l.starts_with("borrow-link: ") && l.contains("= &mut x")));
    }

    #[test]
    fn track_summaries_t2_and_referent_red_edges() {
        let t = parse(DEMO).unwrap();
        let facts = ast::extract(&t);
        let sums = track_summaries(&facts);
        assert_eq!(sums.len(), 3);
        for ts in &sums {
            assert_eq!(ts.omega, ts.chi, "{} 軌 T2:χ=ω(完美圖)", ts.label);
        }
        let referent = sums.last().unwrap();
        assert!(
            !referent.edge_lines.is_empty(),
            "演示樣本 referent 軌必有紅邊"
        );
        assert!(referent.edge_lines.iter().any(|l| l.contains("on x")));
    }

    #[test]
    fn config_env_falls_back_to_defaults_on_garbage() {
        // 未設變量 ⇒ 預設;垃圾值亦回退(暫存覆寫避免污染平行測試)
        std::env::set_var("FUZZ_ITERATIONS", "not-a-number");
        let cfg = FuzzConfig::from_env();
        std::env::remove_var("FUZZ_ITERATIONS");
        assert_eq!(cfg.main_iter, FuzzConfig::default().main_iter);
        assert_eq!(cfg.l8_iter, 300);
    }
}
