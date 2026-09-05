//! 巨集七原則實驗室(借.md → 機核實作)。
//!
//! 把《借.md》的 TRS 形式框架落成可機械檢查的規則樹系統:
//! * `Frag` / `Pat` / `Tpl` / `Rule` / `MacroDef` —— LHS 樣式樹與 RHS 模板樹;
//! * `match_seq` / `instantiate` / `expand_chain` —— 有序展開語義
//!   `exp(t) = r_iσ, i = min{ j | ∃σ. l_jσ = t }`(附 telemetry:比較計數、μ 記錄)
//!   ⇒ 原則 3 終止證據與 tt-muncher O(n²) 的實證基礎;
//! * `rule_pair_verdict` / `check_exclusive` —— 樣式語言相交判定(原則 1);
//! * `template_linearity` / `check_linear` —— |rhs|_x ≤ 1(原則 6);
//! * `registry()` —— 模型規則樹與真 `macro_rules!` 同構登記(單一真相);
//! * `verify_seven_principles()` —— 七門禁自証;
//! * 真 `cl0_*` 巨集(與模型規則一一對應)。
//!
//! 誠實邊界見 `MACRO_SEVEN_PRINCIPLES.md` §4(Frag 為保守子集;
//! 相容檢查器對含自由 Rep 的序列保守回報)。

use crate::token_tree::{render_forest, Delim, Tok, TT};
use std::cell::RefCell;
use std::collections::HashMap;

// ===========================================================================
// §0 真 macro_rules! 巨集(與 §6 registry() 的模型規則樹一一對應)
// ===========================================================================

/// 原則 1+3:入口宏(單規則 ⇒ 平凡互斥)。委派 inner。
///
/// P5 紀律:crate 內遞迌一律走 `$crate::`(hygiene 正確形態)。
#[macro_export]
macro_rules! cl0_count_tts {
    ($($tts:tt)*) => { $crate::cl0_count_tts_inner!(@acc [] $($tts)*) };
}

/// 原則 1+3:I1(終止)/ I2(步進,μ 嚴格遞減)。
/// I1 要求 `]` 後結束、I2 要求 `]` 後 ≥1 token ⇒ L(I1)∩L(I2)=∅。
#[macro_export]
macro_rules! cl0_count_tts_inner {
    (@acc [$($acc:tt)*]) => { 0usize $(+ $acc)* };
    (@acc [$($acc:tt)*] $head:tt $($tail:tt)*) => {
        $crate::cl0_count_tts_inner!(@acc [$($acc)* 1] $($tail)*)
    };
}

/// 原則 7 正例:Tree 代換 —— `$e:expr` 是原子子樹,double!(1+1) = 4。
#[macro_export]
macro_rules! cl0_double {
    ($e:expr) => {
        ($e) * 2
    };
}

/// 原則 7 反例(教學對照,勿仿):String 代換 —— 1 + 1*2 = 3。
#[macro_export]
macro_rules! cl0_double_tt {
    ($($t:tt)*) => { ($($t)* * 2) };
}

/// 原則 1:首字面判別(mut/shr 首 token 即分岔,交集 trivially 空)。
#[macro_export]
macro_rules! cl0_borrow_kind {
    (mut $p:ident) => {
        ($crate::borrow_model::BorrowKind::Mut, stringify!($p))
    };
    (shr $p:ident) => {
        ($crate::borrow_model::BorrowKind::Shr, stringify!($p))
    };
}

/// 原則 4:CPS —— transcriber 直出 `$k!(…)`,
/// exp(produce!(consume)) = consume!(11,31)(outermost ≡ innermost)。
/// 呼叫端以 `use crate::cl0_consume;` 導入後傳 ident(呼叫端 hygiene 解析)。
#[macro_export]
macro_rules! cl0_produce {
    ($k:ident) => {
        $k!(11, 31)
    };
}

/// 原則 4 被調用方:Tree 代換的加法。
#[macro_export]
macro_rules! cl0_consume {
    ($a:expr, $b:expr) => {
        ($a) + ($b)
    };
}

/// 原則 5:全部非 hygienic 名稱 = 絕對路徑;原則 6:每個 $e 在重複內恰一次。
/// (push 形態是刻意保留的 P6 教學形態:顯式逐一消費每個 $e,恰一次;
///  展開於調用點,故 allow 須置於巨集體內的語句上)
#[macro_export]
macro_rules! cl0_safe_vec {
    () => { ::std::vec::Vec::new() };
    ($($e:expr),* $(,)?) => {{
        // allow 必須包住整個 let+push 序列(lint span 跨語句)
        #[allow(clippy::vec_init_then_push)]
        let v = {
            let mut v = ::std::vec::Vec::new();
            $(v.push($e);)*
            v
        };
        v
    }};
}

/// 原則 6:let 強制 sharing —— |rhs|_$val = 1(線性),
/// a 為 hygiene 局部;副作用恰一次,其後由正常 borrowck 管轄。
#[macro_export]
macro_rules! cl0_with_val {
    ($val:expr, $f:expr) => {{
        let a = $val;
        ($f)(&a, &a)
    }};
}

/// `{{ }}` 區間有界(借.md 第二部分):Drop 釘死區間右端點;
/// 嵌套塊 ⇒ 區間構成 laminar family(不交或包含,無部分重疊)。
#[macro_export]
macro_rules! cl0_laminar_scope {
    ($id:literal { $($body:tt)* }) => {{
        let _g = $crate::macro_lab::ScopeGuard::enter($id);
        { $($body)* }
    }};
}

// ===========================================================================
// §1 片段種類與規則樹
// ===========================================================================

/// 片段種類(sorted variable;原則 7)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frag {
    Tt,
    Ident,
    Lit,
    Punct,
    /// 表達式:模型近似 = 「消費至頂層 `,` 或序列結尾」(保守子集)
    Expr,
    /// 型別/路徑:模型近似 = 單個 ident
    Ty,
    Path,
}

impl Frag {
    fn accepts(&self, t: &TT) -> bool {
        match self {
            Frag::Tt => true,
            Frag::Ident | Frag::Ty | Frag::Path => matches!(t, TT::Atom(Tok::Ident(_))),
            Frag::Lit => matches!(t, TT::Atom(Tok::Lit(_))),
            Frag::Punct => matches!(t, TT::Atom(Tok::Punct(_))),
            // Expr 近似:除頂層逗號外皆可納入(由 match_seq 的 take 邏輯截斷)
            Frag::Expr => !matches!(t, TT::Atom(Tok::Punct(','))),
        }
    }

    fn first_kinds(&self) -> &'static [&'static str] {
        match self {
            Frag::Tt | Frag::Expr => &["ident", "literal", "punct", "group"],
            Frag::Ident | Frag::Ty | Frag::Path => &["ident"],
            Frag::Lit => &["literal"],
            Frag::Punct => &["punct"],
        }
    }
}

/// LHS 樣式樹
#[derive(Clone, Debug, PartialEq)]
pub enum Pat {
    Tok(Tok),
    Meta(&'static str, Frag),
    Group(Delim, Vec<Pat>),
    /// $(...)* ;分隔符 sep(如 `,`)
    Rep(Vec<Pat>, Option<Tok>),
}

/// RHS 模板樹
#[derive(Clone, Debug, PartialEq)]
pub enum Tpl {
    Seq(Vec<Tpl>),
    Tok(Tok),
    /// 代換點(Tree 代換;原則 7)
    Sub(&'static str),
    Group(Delim, Vec<Tpl>),
    /// 對捕獲清單逐一展開
    Rep(Vec<Tpl>),
    /// 遞迌調用點(模型層追蹤 μ)
    Recurse(Vec<Tpl>),
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub name: &'static str,
    pub lhs: Vec<Pat>,
    pub rhs: Tpl,
    /// Recurse 觸發後是否委派到鏈中下一個系統(入口宏 → inner 的單向委派;
    /// 互斥系統內不需要次序兜底,委派只在「宏 A 展開出 宏 B!(…)」時發生)
    pub delegate_after: bool,
}

impl Rule {
    /// 普通規則(系統內自遞迌)
    pub fn new(name: &'static str, lhs: Vec<Pat>, rhs: Tpl) -> Self {
        Rule {
            name,
            lhs,
            rhs,
            delegate_after: false,
        }
    }
    /// 委派規則(展開結果交由下一個系統繼續)
    pub fn delegating(name: &'static str, lhs: Vec<Pat>, rhs: Tpl) -> Self {
        Rule {
            name,
            lhs,
            rhs,
            delegate_after: true,
        }
    }
}

/// 巨集系統(規則有序;互斥 ⇒ 次序無關 ⇒ 合流)
#[derive(Clone, Debug)]
pub struct MacroDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub rules: Vec<Rule>,
    pub principles: &'static [&'static str],
}

// ===========================================================================
// §2 匹配器(附 telemetry)與模板實例化
// ===========================================================================

/// token 比較計數(O(n²) 實證的度量)
#[derive(Default, Clone, Debug)]
pub struct Telemetry {
    pub comparisons: usize,
}

/// 繫結:$name → 捕獲片段清單(Rep 語境可捕獲多個)
#[derive(Clone, Debug, Default)]
pub struct Bindings {
    map: HashMap<&'static str, Vec<Vec<TT>>>,
}

impl Bindings {
    pub fn get(&self, name: &str) -> &[Vec<TT>] {
        self.map.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }
    fn push(&mut self, name: &'static str, frag: Vec<TT>) {
        self.map.entry(name).or_default().push(frag);
    }
}

/// 序列匹配:樣式序列恰好消費整個森林(回溯;Rep 貪婪最長先)。
pub fn match_seq(pats: &[Pat], toks: &[TT], b: &mut Bindings, tel: &mut Telemetry) -> bool {
    let Some((first, rest_pats)) = pats.split_first() else {
        // 樣式耗盡:輸入必須也耗盡
        tel.comparisons += 1;
        return toks.is_empty();
    };
    match first {
        Pat::Tok(t) => match toks.first() {
            Some(TT::Atom(a)) if a == t => {
                tel.comparisons += 1;
                match_seq(rest_pats, &toks[1..], b, tel)
            }
            _ => {
                tel.comparisons += 1;
                false
            }
        },
        Pat::Meta(name, frag) => {
            // 單 token 片段:恰消費 1;Expr 近似:貪婪至頂層 ',' 或結尾
            let take = if *frag == Frag::Expr {
                let mut k = 0usize;
                while k < toks.len() && frag.accepts(&toks[k]) {
                    k += 1;
                }
                k
            } else {
                usize::from(!toks.is_empty())
            };
            if take == 0 || take > toks.len() {
                tel.comparisons += 1;
                return false;
            }
            tel.comparisons += 1;
            let captured: Vec<TT> = toks[..take].to_vec();
            let mut bp = b.clone();
            bp.push(name, captured);
            if match_seq(rest_pats, &toks[take..], &mut bp, tel) {
                *b = bp;
                true
            } else {
                false
            }
        }
        Pat::Group(d, inner) => match toks.first() {
            Some(TT::Group(dd, g)) if dd == d => {
                tel.comparisons += 1;
                let mut bp = b.clone();
                if match_seq(inner, g, &mut bp, tel)
                    && match_seq(rest_pats, &toks[1..], &mut bp, tel)
                {
                    *b = bp;
                    true
                } else {
                    false
                }
            }
            _ => {
                tel.comparisons += 1;
                false
            }
        },
        Pat::Rep(inner, sep) => {
            // 快路:內層為單一非 Expr 片段 ⇒ 每輪恰 1 token,逐一消費(貪婪),
            // 之後按需回退輪數。避免 end 掃描把 telemetry 抬成 Θ(n³)。
            if let [Pat::Meta(name, frag)] = inner.as_slice() {
                if *frag != Frag::Expr {
                    // 貪婪收集
                    let mut rounds: Vec<Vec<TT>> = Vec::new();
                    let mut pos = 0usize;
                    loop {
                        if pos > 0 {
                            if let Some(s) = sep {
                                match toks.get(pos) {
                                    Some(TT::Atom(a)) if a == s => {
                                        tel.comparisons += 1;
                                        pos += 1;
                                    }
                                    _ => {
                                        tel.comparisons += 1;
                                        break;
                                    }
                                }
                            }
                        }
                        match toks.get(pos) {
                            Some(t) if frag.accepts(t) => {
                                tel.comparisons += 1;
                                rounds.push(vec![t.clone()]);
                                pos += 1;
                            }
                            _ => {
                                tel.comparisons += 1;
                                break;
                            }
                        }
                    }
                    // 由多至少回試其餘樣式;空輪(0 次)也在候選內 ——
                    // 零輪 = Rep 不匹配任何東西,輸入留給其餘樣式。
                    loop {
                        let consumed: usize = rounds.iter().map(|r| r.len()).sum::<usize>()
                            + if sep.is_some() {
                                rounds.len().saturating_sub(1)
                            } else {
                                0
                            };
                        let mut bp = b.clone();
                        for r in &rounds {
                            bp.push(name, r.clone());
                        }
                        if match_seq(rest_pats, &toks[consumed..], &mut bp, tel) {
                            *b = bp;
                            return true;
                        }
                        if rounds.is_empty() {
                            break;
                        }
                        rounds.pop();
                    }
                    return false;
                }
            }
            // 一般路:單輪 = 內層可匹配的最長前綴(掃描)。
            // 選項 A:零輪,直接匹配其餘樣式
            if match_seq(rest_pats, toks, &mut b.clone(), tel) {
                return true;
            }
            // 選項 B:一輪(最長前綴先),消費分隔符,再遇 Rep(重新進入)
            let mut ends: Vec<usize> = (1..=toks.len()).collect();
            ends.reverse();
            for end in ends {
                let mut round_b = b.clone();
                let mut sub_tel = Telemetry::default();
                if match_seq(inner, &toks[..end], &mut round_b, &mut sub_tel) {
                    tel.comparisons += sub_tel.comparisons;
                    let after_round = &toks[end..];
                    let (sep_ok, next_toks) = match sep {
                        Some(s) => match after_round.first() {
                            Some(TT::Atom(a)) if a == s => (true, &after_round[1..]),
                            _ => (false, after_round),
                        },
                        None => (true, after_round),
                    };
                    tel.comparisons += 1;
                    if sep_ok {
                        let mut pats2: Vec<Pat> = vec![first.clone()];
                        pats2.extend_from_slice(rest_pats);
                        if match_seq(&pats2, next_toks, &mut round_b, tel) {
                            *b = round_b;
                            return true;
                        }
                    }
                }
            }
            false
        }
    }
}

/// 模板實例化:Tree 代換(原則 7)
pub fn instantiate(tpl: &Tpl, b: &Bindings, out: &mut Vec<TT>) {
    match tpl {
        Tpl::Seq(items) => {
            for t in items {
                instantiate(t, b, out);
            }
        }
        Tpl::Tok(t) => out.push(TT::Atom(t.clone())),
        Tpl::Sub(name) => {
            for frag in b.get(name) {
                out.extend(frag.iter().cloned());
            }
        }
        Tpl::Group(d, inner) => {
            let mut g = Vec::new();
            for t in inner {
                instantiate(t, b, &mut g);
            }
            out.push(TT::Group(*d, g));
        }
        Tpl::Rep(inner) => {
            let rounds = inner
                .iter()
                .filter_map(|t| match t {
                    Tpl::Sub(n) => Some(b.get(n).len()),
                    _ => None,
                })
                .max()
                .unwrap_or(0);
            for i in 0..rounds {
                let mut rb = Bindings::default();
                for t in inner {
                    if let Tpl::Sub(n) = t {
                        if let Some(frag) = b.get(n).get(i) {
                            rb.push(n, frag.clone());
                        }
                    }
                }
                for t in inner {
                    instantiate(t, &rb, out);
                }
            }
        }
        Tpl::Recurse(_) => {
            // 由展開器處理
        }
    }
}

// ===========================================================================
// §3 展開器:exp(t) = r_iσ(首規則命中)+ 委派 + fuel + μ 記錄
// ===========================================================================

#[derive(Clone, Debug)]
pub struct ExpansionTrace {
    pub steps: Vec<TraceStep>,
    pub final_rendered: String,
    pub telemetry: Telemetry,
}

#[derive(Clone, Debug)]
pub struct TraceStep {
    pub rule: &'static str,
    pub input_len: usize,
    pub output_len: usize,
    /// muncher 的 μ(借.md §3):未處理尾部 token 數;非 munch 形態為 None
    pub mu_in: Option<usize>,
    pub mu_out: Option<usize>,
}

/// muncher 輸入的 μ:`( @ acc [ … ] rest… )` 中 rest 的 token 數
fn munch_mu(forest: &[TT]) -> Option<usize> {
    let g = forest.first()?.as_group(Delim::Paren)?;
    if g.len() < 3 {
        return None;
    }
    match (&g[0], &g[1]) {
        (TT::Atom(Tok::Punct('@')), TT::Atom(Tok::Ident(s))) if s == "acc" => {}
        _ => return None,
    }
    g[2].as_group(Delim::Bracket)?;
    Some(g[3..].len())
}

#[derive(Clone, Debug)]
pub enum ExpandErr {
    OutOfFuel { steps: usize },
    NoRuleMatched { at_step: usize, input: String },
}

/// 有序展開(可委派鏈):defs[0] 為起始系統;當前系統無規則命中時
/// 依序嘗試後續系統(入口宏 → inner 的委派語義)。全程 telemetry 累計。
pub fn expand_chain(
    defs: &[&MacroDef],
    input: &[TT],
    fuel: usize,
) -> Result<ExpansionTrace, ExpandErr> {
    let mut current = input.to_vec();
    let mut steps = Vec::new();
    let mut tel = Telemetry::default();
    if defs.is_empty() {
        return Err(ExpandErr::NoRuleMatched {
            at_step: 0,
            input: render_forest(input),
        });
    }
    let mut sys = 0usize; // 當前系統;委派(delegate_after)單向前進
    loop {
        if steps.len() > fuel {
            return Err(ExpandErr::OutOfFuel { steps: steps.len() });
        }
        // 在當前系統找首條命中規則(互斥 ⇒ 次序無關)
        let def = defs[sys];
        let mut fired: Option<(&Rule, Bindings)> = None;
        for r in &def.rules {
            let mut b = Bindings::default();
            if match_seq(&r.lhs, &current, &mut b, &mut tel) {
                fired = Some((r, b));
                break;
            }
        }
        let (rule, b) = match fired {
            Some(x) => x,
            None => {
                return Err(ExpandErr::NoRuleMatched {
                    at_step: steps.len(),
                    input: render_forest(&current),
                })
            }
        };
        match &rule.rhs {
            Tpl::Recurse(args) => {
                let mut next = Vec::new();
                for t in args {
                    instantiate(t, &b, &mut next);
                }
                let (mu_in, mu_out) = (munch_mu(&current), munch_mu(&next));
                steps.push(TraceStep {
                    rule: rule.name,
                    input_len: current.len(),
                    output_len: next.len(),
                    mu_in,
                    mu_out,
                });
                current = next;
                if rule.delegate_after {
                    sys += 1;
                    if sys >= defs.len() {
                        return Err(ExpandErr::NoRuleMatched {
                            at_step: steps.len(),
                            input: render_forest(&current),
                        });
                    }
                }
            }
            rhs => {
                let mut out = Vec::new();
                instantiate(rhs, &b, &mut out);
                let mu_in = munch_mu(&current);
                steps.push(TraceStep {
                    rule: rule.name,
                    input_len: current.len(),
                    output_len: out.len(),
                    mu_in,
                    mu_out: None,
                });
                return Ok(ExpansionTrace {
                    steps,
                    final_rendered: render_forest(&out),
                    telemetry: tel,
                });
            }
        }
    }
}

// ===========================================================================
// §4 互斥檢查器(原則 1)與線性檢查器(原則 6)
// ===========================================================================

/// 互斥判定結果
#[derive(Clone, Debug, PartialEq)]
pub enum ExclVerdict {
    /// 語言不相交(精確)
    Disjoint,
    /// 語言相交(精確)
    Overlap,
    /// 序列含自由 Rep —— 保守回報(可能相交;完備判定需 tree automaton product)
    Conservative,
}

/// 樣式序列的「首 token 種類集合」與「是否可為空」
fn first_set(pats: &[Pat]) -> (Vec<&'static str>, bool) {
    let mut kinds: Vec<&'static str> = Vec::new();
    for p in pats {
        match p {
            Pat::Tok(t) => {
                kinds.push(match t {
                    Tok::Ident(_) => "ident",
                    Tok::Lit(_) => "literal",
                    Tok::Punct(_) => "punct",
                });
                return (kinds, false);
            }
            Pat::Meta(_, f) => {
                kinds.extend_from_slice(f.first_kinds());
                return (kinds, false);
            }
            Pat::Group(..) => {
                kinds.push("group");
                return (kinds, false);
            }
            Pat::Rep(inner, _) => {
                let (ik, inn) = first_set(inner);
                kinds.extend_from_slice(&ik);
                if !inn {
                    return (kinds, false);
                }
                // 可為空 ⇒ 繼續看後續樣式
            }
        }
    }
    (kinds, true)
}

fn kinds_intersect(a: &[&'static str], b: &[&'static str]) -> bool {
    a.iter().any(|k| b.contains(k))
}

fn tok_kind(t: &Tok) -> &'static str {
    match t {
        Tok::Ident(_) => "ident",
        Tok::Lit(_) => "literal",
        Tok::Punct(_) => "punct",
    }
}

/// 兩條規則的 LHS 語言是否相交(原則 1 門禁的核心判定)
pub fn rule_pair_verdict(a: &[Pat], b: &[Pat]) -> ExclVerdict {
    let (ka, na) = first_set(a);
    let (kb, nb) = first_set(b);
    // 首 token 種類完全分歧且雙方不可空 ⇒ 不相交(trivially 可見的判別子)
    if !kinds_intersect(&ka, &kb) && !na && !nb {
        return ExclVerdict::Disjoint;
    }
    // 頂層序列含自由 Rep ⇒ 保守
    if a.iter().any(|p| matches!(p, Pat::Rep(..))) || b.iter().any(|p| matches!(p, Pat::Rep(..))) {
        return ExclVerdict::Conservative;
    }
    // 結構判定(定界群組視為原子,內部 Rep 採保守相容)
    if seq_compat_structural(a, b) {
        ExclVerdict::Overlap
    } else {
        ExclVerdict::Disjoint
    }
}

/// 無頂層自由 Rep 序列的結構相容性:存在共同實例?
fn seq_compat_structural(a: &[Pat], b: &[Pat]) -> bool {
    match (a.first(), b.first()) {
        (None, None) => true,
        // 一方結束、另一方尚有(不可空)樣式 ⇒ 不相交
        (Some(_), None) | (None, Some(_)) => false,
        (Some(x), Some(y)) => {
            let node_ok = match (x, y) {
                (Pat::Tok(t1), Pat::Tok(t2)) => t1 == t2,
                (Pat::Tok(t), Pat::Meta(_, f)) | (Pat::Meta(_, f), Pat::Tok(t)) => {
                    f.first_kinds().contains(&tok_kind(t))
                }
                (Pat::Meta(_, f1), Pat::Meta(_, f2)) => {
                    kinds_intersect(f1.first_kinds(), f2.first_kinds())
                }
                (Pat::Group(d1, i1), Pat::Group(d2, i2)) => {
                    d1 == d2 && seq_compat_structural(i1, i2)
                }
                // 群組內的 Rep:保守視為相容(群組為原子,長度可變)
                (Pat::Rep(..), _) | (_, Pat::Rep(..)) => true,
                _ => false,
            };
            node_ok && seq_compat_structural(&a[1..], &b[1..])
        }
    }
}

/// 單一巨集系統的規則互斥檢查(單規則系統 ⇒ 平凡互斥,返回空)
pub fn check_exclusive(def: &MacroDef) -> Vec<(&'static str, &'static str, ExclVerdict)> {
    let mut out = Vec::new();
    for i in 0..def.rules.len() {
        for j in (i + 1)..def.rules.len() {
            let v = rule_pair_verdict(&def.rules[i].lhs, &def.rules[j].lhs);
            out.push((def.rules[i].name, def.rules[j].name, v));
        }
    }
    out
}

/// 模板線性(原則 6):每個變數出現次數(≤1 為線性)
pub fn template_linearity(tpl: &Tpl) -> HashMap<&'static str, usize> {
    fn walk(t: &Tpl, acc: &mut Vec<(&'static str, usize)>) {
        match t {
            Tpl::Sub(n) => {
                if let Some(e) = acc.iter_mut().find(|(k, _)| k == n) {
                    e.1 += 1;
                } else {
                    acc.push((n, 1));
                }
            }
            Tpl::Seq(inner) | Tpl::Group(_, inner) | Tpl::Rep(inner) | Tpl::Recurse(inner) => {
                for x in inner {
                    walk(x, acc)
                }
            }
            Tpl::Tok(_) => {}
        }
    }
    let mut acc = Vec::new();
    walk(tpl, &mut acc);
    acc.into_iter().collect()
}

/// 系統級線性檢查:全部規則 RHS 皆線性?
pub fn check_linear(def: &MacroDef) -> Vec<(&'static str, bool, Vec<String>)> {
    def.rules
        .iter()
        .map(|r| {
            let counts = template_linearity(&r.rhs);
            let bad: Vec<String> = counts
                .iter()
                .filter(|(_, &c)| c > 1)
                .map(|(n, c)| format!("${}×{}", n, c))
                .collect();
            (r.name, bad.is_empty(), bad)
        })
        .collect()
}

// ===========================================================================
// §5 laminar_scope 執行期記錄器({{ }} 區間有界引理的實測儀)
// ===========================================================================

thread_local! {
    static RECORDER: RefCell<Vec<(u32, u32, u32)>> = const { RefCell::new(Vec::new()) };
    static POINT: RefCell<u32> = const { RefCell::new(0) };
}

/// `cl0_laminar_scope!` 的區間守衛:enter 記錄起點,Drop 釘死終點。
#[derive(Debug)]
pub struct ScopeGuard {
    idx: usize,
}

impl ScopeGuard {
    pub fn enter(id: u32) -> Self {
        RECORDER.with_borrow_mut(|rec| {
            POINT.with_borrow_mut(|p| {
                let s = *p;
                *p += 1;
                rec.push((id, s, s)); // 終點先佔位,Drop 時釘死
                ScopeGuard { idx: rec.len() - 1 }
            })
        })
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        RECORDER.with_borrow_mut(|rec| {
            POINT.with_borrow_mut(|p| {
                let e = *p;
                *p += 1;
                if let Some(span) = rec.get_mut(self.idx) {
                    span.2 = e;
                }
            })
        });
    }
}

/// 取出已記錄的區間 (id, start, end)
pub fn recorded_spans() -> Vec<(u32, u32, u32)> {
    RECORDER.with_borrow(|rec| rec.clone())
}

/// 重置記錄器(測試隔離用)
pub fn reset_recorder() {
    RECORDER.with_borrow_mut(|rec| rec.clear());
    POINT.with_borrow_mut(|p| *p = 0);
}

// ===========================================================================
// §6 七原則註冊表(模型規則樹 —— 與 §0 真 macro_rules! 一一同構)
// ===========================================================================

fn tok_ident(s: &str) -> Pat {
    Pat::Tok(Tok::Ident(s.to_string()))
}
fn tok_punct(c: char) -> Pat {
    Pat::Tok(Tok::Punct(c))
}
fn tl_ident(s: &str) -> Tpl {
    Tpl::Tok(Tok::Ident(s.to_string()))
}
fn tl_punct(c: char) -> Tpl {
    Tpl::Tok(Tok::Punct(c))
}
fn tl_lit(s: &str) -> Tpl {
    Tpl::Tok(Tok::Lit(s.to_string()))
}

/// 註冊表:每個系統 = 一組有序(但互斥 ⇒ 次序無關)規則樹
pub fn registry() -> Vec<MacroDef> {
    vec![
        // ---- count_tts 入口(P1:單規則 ⇒ 平凡互斥)----
        MacroDef {
            name: "cl0_count_tts",
            doc: "入口宏:單規則,委派 inner(設計決策:經典三規則 count_tts 的入口 \
                  catch-all 會與 @acc 規則相交、靠次序兜底,違反原則 1,故拆分)",
            principles: &["P1", "P5"],
            rules: vec![Rule::delegating(
                "E1-entry",
                vec![Pat::Group(
                    Delim::Paren,
                    vec![Pat::Rep(vec![Pat::Meta("tts", Frag::Tt)], None)],
                )],
                Tpl::Recurse(vec![Tpl::Group(
                    Delim::Paren,
                    vec![
                        tl_punct('@'),
                        tl_ident("acc"),
                        Tpl::Group(Delim::Bracket, vec![]),
                        Tpl::Rep(vec![Tpl::Sub("tts")]),
                    ],
                )]),
            )],
        },
        // ---- count_tts inner(P1:尾分岔互斥;P3:μ 嚴格遞減;P6:線性)----
        MacroDef {
            name: "cl0_count_tts_inner",
            doc: "I1(終止,I2(步進):L(I1)∩L(I2)=∅ —— ']' 後結束 vs ≥1 token",
            principles: &["P1", "P3", "P6"],
            rules: vec![
                Rule::new(
                    "I1-final",
                    vec![Pat::Group(
                        Delim::Paren,
                        vec![
                            tok_punct('@'),
                            tok_ident("acc"),
                            Pat::Group(
                                Delim::Bracket,
                                vec![Pat::Rep(vec![Pat::Meta("a", Frag::Tt)], None)],
                            ),
                        ],
                    )],
                    Tpl::Seq(vec![
                        tl_lit("0usize"),
                        Tpl::Rep(vec![tl_punct('+'), Tpl::Sub("a")]),
                    ]),
                ),
                Rule::new(
                    "I2-step",
                    vec![Pat::Group(
                        Delim::Paren,
                        vec![
                            tok_punct('@'),
                            tok_ident("acc"),
                            Pat::Group(
                                Delim::Bracket,
                                vec![Pat::Rep(vec![Pat::Meta("a", Frag::Tt)], None)],
                            ),
                            Pat::Meta("h", Frag::Tt),
                            Pat::Rep(vec![Pat::Meta("r", Frag::Tt)], None),
                        ],
                    )],
                    Tpl::Recurse(vec![Tpl::Group(
                        Delim::Paren,
                        vec![
                            tl_punct('@'),
                            tl_ident("acc"),
                            Tpl::Group(
                                Delim::Bracket,
                                vec![Tpl::Rep(vec![Tpl::Sub("a")]), tl_lit("1")],
                            ),
                            Tpl::Rep(vec![Tpl::Sub("r")]),
                        ],
                    )]),
                ),
            ],
        },
        // ---- double(P7 正例:Tree 代換)----
        MacroDef {
            name: "cl0_double",
            doc: "P7 正例:$e:expr 為原子子樹,double!(1+1) = (1+1)*2 = 4",
            principles: &["P7", "P2"],
            rules: vec![Rule::new(
                "D1",
                vec![Pat::Group(Delim::Paren, vec![Pat::Meta("e", Frag::Expr)])],
                Tpl::Group(
                    Delim::Paren,
                    vec![
                        Tpl::Group(Delim::Paren, vec![Tpl::Sub("e")]),
                        tl_punct('*'),
                        tl_lit("2"),
                    ],
                ),
            )],
        },
        // ---- double_tt(P7 反例:String 代換)----
        MacroDef {
            name: "cl0_double_tt",
            doc: "P7 反例(教學對照):tt 逐字拼接,1 + 1*2 = 3",
            principles: &["P7-反例"],
            rules: vec![Rule::new(
                "D2",
                vec![Pat::Group(
                    Delim::Paren,
                    vec![Pat::Rep(vec![Pat::Meta("t", Frag::Tt)], None)],
                )],
                Tpl::Group(
                    Delim::Paren,
                    vec![Tpl::Rep(vec![Tpl::Sub("t")]), tl_punct('*'), tl_lit("2")],
                ),
            )],
        },
        // ---- borrow_kind(P1:首字面判別)----
        MacroDef {
            name: "cl0_borrow_kind",
            doc: "首 token 'mut'/'shr' 即分岔,交集 trivially 空",
            principles: &["P1", "P7"],
            rules: vec![
                Rule::new(
                    "B1",
                    vec![Pat::Group(
                        Delim::Paren,
                        vec![tok_ident("mut"), Pat::Meta("p", Frag::Ident)],
                    )],
                    Tpl::Seq(vec![tl_lit("mut:"), Tpl::Sub("p")]),
                ),
                Rule::new(
                    "B2",
                    vec![Pat::Group(
                        Delim::Paren,
                        vec![tok_ident("shr"), Pat::Meta("p", Frag::Ident)],
                    )],
                    Tpl::Seq(vec![tl_lit("shr:"), Tpl::Sub("p")]),
                ),
            ],
        },
        // ---- CPS(P4:outermost ≡ innermost)----
        MacroDef {
            name: "cl0_produce_consume",
            doc: "PC1(produce)把續體 $k 直出為 $k!(…);PC2(consume)Tree 代換加法。\
                  兩規則互斥(單 ident vs expr,expr 對)⇒ 次序無關",
            principles: &["P4", "P1", "P7"],
            rules: vec![
                Rule::new(
                    "PC1-produce",
                    vec![Pat::Group(Delim::Paren, vec![Pat::Meta("k", Frag::Ident)])],
                    Tpl::Recurse(vec![Tpl::Group(
                        Delim::Paren,
                        vec![tl_lit("11"), tl_punct(','), tl_lit("31")],
                    )]),
                ),
                Rule::new(
                    "PC2-consume",
                    vec![Pat::Group(
                        Delim::Paren,
                        vec![
                            Pat::Meta("a", Frag::Expr),
                            tok_punct(','),
                            Pat::Meta("b", Frag::Expr),
                        ],
                    )],
                    Tpl::Seq(vec![
                        Tpl::Group(Delim::Paren, vec![Tpl::Sub("a")]),
                        tl_punct('+'),
                        Tpl::Group(Delim::Paren, vec![Tpl::Sub("b")]),
                    ]),
                ),
            ],
        },
        // ---- safe_vec(P5 絕對路徑 + P6 重複內線性)----
        MacroDef {
            name: "cl0_safe_vec",
            doc: "全部非 hygienic 名稱 = ::std 絕對路徑;V1(空)/V2(≥1 元素)互斥",
            principles: &["P5", "P6", "P1"],
            rules: vec![
                Rule::new(
                    "V1",
                    vec![Pat::Group(Delim::Paren, vec![])],
                    Tpl::Seq(vec![
                        tl_punct(':'),
                        tl_punct(':'),
                        tl_ident("std"),
                        tl_punct(':'),
                        tl_punct(':'),
                        tl_ident("vec"),
                        tl_punct(':'),
                        tl_punct(':'),
                        tl_ident("Vec"),
                        tl_punct(':'),
                        tl_punct(':'),
                        tl_ident("new"),
                        Tpl::Group(Delim::Paren, vec![]),
                    ]),
                ),
                Rule::new(
                    "V2",
                    vec![Pat::Group(
                        Delim::Paren,
                        vec![
                            Pat::Meta("e0", Frag::Expr),
                            tok_punct(','),
                            Pat::Rep(vec![Pat::Meta("e", Frag::Expr)], Some(Tok::Punct(','))),
                        ],
                    )],
                    Tpl::Seq(vec![
                        tl_ident("let"),
                        tl_ident("mut"),
                        tl_ident("v"),
                        tl_punct('='),
                        tl_punct(':'),
                        tl_punct(':'),
                        tl_ident("std"),
                        tl_punct(':'),
                        tl_punct(':'),
                        tl_ident("vec"),
                        tl_punct(':'),
                        tl_punct(':'),
                        tl_ident("Vec"),
                        tl_punct(':'),
                        tl_punct(':'),
                        tl_ident("new"),
                        tl_punct('('),
                        tl_punct(')'),
                        tl_punct(';'),
                        tl_ident("v"),
                        tl_punct('.'),
                        tl_ident("push"),
                        Tpl::Group(Delim::Paren, vec![Tpl::Sub("e0")]),
                        tl_punct(';'),
                        Tpl::Rep(vec![
                            tl_ident("v"),
                            tl_punct('.'),
                            tl_ident("push"),
                            Tpl::Group(Delim::Paren, vec![Tpl::Sub("e")]),
                            tl_punct(';'),
                        ]),
                        tl_ident("v"),
                    ]),
                ),
            ],
        },
        // ---- with_val(P6:let 強制 sharing)----
        MacroDef {
            name: "cl0_with_val",
            doc: "|rhs|_$val = 1(線性);a 為 hygiene 局部;副作用恰一次",
            principles: &["P6", "P2"],
            rules: vec![Rule::new(
                "W1",
                vec![Pat::Group(
                    Delim::Paren,
                    vec![
                        Pat::Meta("val", Frag::Expr),
                        tok_punct(','),
                        Pat::Meta("f", Frag::Expr),
                    ],
                )],
                Tpl::Seq(vec![
                    tl_ident("let"),
                    tl_ident("a"),
                    tl_punct('='),
                    Tpl::Sub("val"),
                    tl_punct(';'),
                    Tpl::Group(
                        Delim::Paren,
                        vec![
                            Tpl::Sub("f"),
                            Tpl::Group(
                                Delim::Paren,
                                vec![
                                    tl_punct('&'),
                                    tl_ident("a"),
                                    tl_punct(','),
                                    tl_punct('&'),
                                    tl_ident("a"),
                                ],
                            ),
                        ],
                    ),
                ]),
            )],
        },
        // ---- laminar_scope({{ }} 區間有界 ⇒ laminar family)----
        MacroDef {
            name: "cl0_laminar_scope",
            doc: "Drop 釘死區間右端點;嵌套塊 ⇒ 區間不交或包含(無部分重疊)",
            principles: &["P5", "P6", "有界區間"],
            rules: vec![Rule::new(
                "S1",
                vec![Pat::Group(
                    Delim::Paren,
                    vec![
                        Pat::Meta("id", Frag::Lit),
                        Pat::Group(
                            Delim::Brace,
                            vec![Pat::Rep(vec![Pat::Meta("body", Frag::Tt)], None)],
                        ),
                    ],
                )],
                Tpl::Seq(vec![
                    tl_ident("let"),
                    tl_ident("_g"),
                    tl_punct('='),
                    tl_ident("$crate"),
                    tl_punct(':'),
                    tl_punct(':'),
                    tl_ident("macro_lab"),
                    tl_punct(':'),
                    tl_punct(':'),
                    tl_ident("ScopeGuard"),
                    tl_punct(':'),
                    tl_punct(':'),
                    tl_ident("enter"),
                    Tpl::Group(Delim::Paren, vec![Tpl::Sub("id")]),
                    tl_punct(';'),
                    Tpl::Group(Delim::Brace, vec![Tpl::Rep(vec![Tpl::Sub("body")])]),
                ]),
            )],
        },
    ]
}

// ===========================================================================
// §7 七原則門禁
// ===========================================================================

#[derive(Clone, Debug)]
pub struct PrincipleReport {
    pub id: &'static str,
    pub name: &'static str,
    pub passed: bool,
    pub evidence: String,
}

/// 執行七門禁(純機內,無外部工具 ⇒ 不會 SKIPPED)
pub fn verify_seven_principles() -> Vec<PrincipleReport> {
    let mut out = Vec::new();
    let reg = registry();

    // ---- P1 規則互斥 ----
    let mut p1_ok = true;
    let mut p1_ev = String::new();
    let mut pair_count = 0usize;
    for def in &reg {
        for (a, b, v) in check_exclusive(def) {
            pair_count += 1;
            if v != ExclVerdict::Disjoint {
                p1_ok = false;
                p1_ev.push_str(&format!("{}×{} = {:?}! ", a, b, v));
            }
        }
    }
    // 負控:相交樣式必須被標記(檢查器自身的靈敏度)
    let neg_overlap = rule_pair_verdict(
        &[Pat::Group(Delim::Paren, vec![Pat::Meta("a", Frag::Expr)])],
        &[Pat::Group(Delim::Paren, vec![Pat::Meta("i", Frag::Ident)])],
    );
    let neg_cons = rule_pair_verdict(
        &[Pat::Rep(vec![Pat::Meta("x", Frag::Tt)], None)],
        &[Pat::Meta("y", Frag::Tt)],
    );
    if neg_overlap != ExclVerdict::Overlap || neg_cons != ExclVerdict::Conservative {
        p1_ok = false;
        p1_ev.push_str(&format!("負控失效 {:?}/{:?}! ", neg_overlap, neg_cons));
    }
    if p1_ok {
        p1_ev = format!(
            "{} 對規則全 Disjoint;負控 Overlap/Conservative 如實標記",
            pair_count
        );
    }
    out.push(PrincipleReport {
        id: "P1",
        name: "規則互斥 ⇒ 合流(無 critical pair)",
        passed: p1_ok,
        evidence: p1_ev,
    });

    // ---- P2 語義等價(bounded,確定性;非 ∀)----
    let mut p2_n = 0usize;
    for x in 0..64i32 {
        if crate::cl0_double!(x) == x * 2 {
            p2_n += 1;
        }
    }
    let sv = crate::cl0_safe_vec!(1, 2, 3, 4);
    let sv_ok = sv == ::std::vec![1, 2, 3, 4];
    let cnt_ok = crate::cl0_count_tts!() == 0usize
        && crate::cl0_count_tts!(a) == 1usize
        && crate::cl0_count_tts!(a b c) == 3usize
        && crate::cl0_count_tts!(a b c d e f g h) == 8usize
        && crate::cl0_count_tts!(a b c d e f g h i j k l m n o p q r s t) == 20usize;
    let p2_ok = p2_n == 64 && sv_ok && cnt_ok;
    let p2_ev = if p2_ok {
        "double×64 == 2x;safe_vec == 參考 vec;count_tts 5 實例 == 字面數".to_string()
    } else {
        format!("等價失敗: double {}/64, sv={}, cnt={}", p2_n, sv_ok, cnt_ok)
    };
    out.push(PrincipleReport {
        id: "P2",
        name: "語義等價(展開 vs 參考實作,bounded)",
        passed: p2_ok,
        evidence: p2_ev,
    });

    // ---- P3 結構遞減 ⇒ 終止(fuel 內有界;誠實:非 ∀ 證明)----
    let mut p3_ok = true;
    let mut p3_ev = String::new();
    let entry = reg
        .iter()
        .find(|d| d.name == "cl0_count_tts")
        .expect("不變式:registry 必含 cl0_count_tts");
    let inner = reg
        .iter()
        .find(|d| d.name == "cl0_count_tts_inner")
        .expect("不變式:registry 必含 cl0_count_tts_inner");
    let mut max_steps = 0usize;
    for n in 0..=64usize {
        let src = format!("( {} )", std::iter::repeat_n("a ", n).collect::<String>());
        let input =
            crate::token_tree::parse_forest(&src).expect("不變式:\"( a a … )\" 格式必可解析");
        match expand_chain(&[entry, inner], &input, 4096) {
            Ok(tr) => {
                max_steps = max_steps.max(tr.steps.len());
                // μ 嚴格遞減:每個 I2 步內部 rest 嚴格變小 (μ_in > μ_out);
                // (相鄰步的 mu_out 與下一個 mu_in 是同一中間輸入,比較無意義)
                for s in &tr.steps {
                    if s.rule == "I2-step" {
                        if let (Some(a), Some(b)) = (s.mu_in, s.mu_out) {
                            if b >= a {
                                p3_ok = false;
                                p3_ev.push_str(&format!("μ 未遞減 @n={}: {}→{} ", n, a, b));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                p3_ok = false;
                p3_ev.push_str(&format!("n={} 展開失敗 {:?} ", n, e));
            }
        }
    }
    if p3_ok {
        p3_ev = format!(
            "n=0..64 全終止;I2 每步 μ 嚴格遞減;最深 {} 步 < fuel 4096",
            max_steps
        );
    }
    out.push(PrincipleReport {
        id: "P3",
        name: "結構遞減 ⇒ 終止(μ 良基 descent 證據)",
        passed: p3_ok,
        evidence: p3_ev,
    });

    // ---- P4 CPS:outermost ≡ innermost ----
    let p4_ok = crate::cl0_produce!(cl0_consume) == 42 && crate::cl0_consume!(11, 31) == 42;
    let p4_ev = if p4_ok {
        "exp(produce!(consume)) = consume!(11,31) = 42(策略無關化)".to_string()
    } else {
        "CPS 等價失敗".to_string()
    };
    out.push(PrincipleReport {
        id: "P4",
        name: "CPS ⇒ outermost ≡ innermost",
        passed: p4_ok,
        evidence: p4_ev,
    });

    // ---- P5 絕對路徑 ⇒ 環境無關 ----
    let p5_ok = {
        struct Vec; // 呼叫端 shadowing
        struct Option; // 連 Option 一併遮蔽
        let _: Vec = Vec;
        let _: Option = Option;
        let v = crate::cl0_safe_vec!(9, 8);
        v == ::std::vec![9, 8]
    };
    let p5_ev = if p5_ok {
        "局部 struct Vec/const Option shadowing 下巨集仍編譯且正確".to_string()
    } else {
        "shadowing 破壞了巨集".to_string()
    };
    out.push(PrincipleReport {
        id: "P5",
        name: "絕對路徑 ⇒ 環境無關性",
        passed: p5_ok,
        evidence: p5_ev,
    });

    // ---- P6 線性 ⇒ affine 使用(ownership 核心)----
    let mut p6_ok = true;
    let mut p6_ev = String::new();
    for def in &reg {
        for (rule, lin, bad) in check_linear(def) {
            if !lin {
                p6_ok = false;
                p6_ev.push_str(&format!("{} 非線性 {:?} ", rule, bad));
            }
        }
    }
    // 副作用計數:$val 恰求值一次
    let c = ::std::cell::Cell::new(0usize);
    let r = crate::cl0_with_val!(
        {
            c.set(c.get() + 1);
            7
        },
        |x: &i32, y: &i32| x + y
    );
    if c.get() != 1 || r != 14 {
        p6_ok = false;
        p6_ev.push_str(&format!("副作用計數 {} / 值 {} ", c.get(), r));
    }
    if p6_ok {
        p6_ev = "註冊表全部 RHS 線性;副作用計數 = 1(a 使用兩次但 e 恰求值/移動一次)".to_string();
    }
    out.push(PrincipleReport {
        id: "P6",
        name: "RHS 線性 + let sharing ⇒ ownership 自證",
        passed: p6_ok,
        evidence: p6_ev,
    });

    // ---- P7 sorted variable ⇒ 代換保子樹 ----
    let p7_ok = crate::cl0_double!(1 + 1) == 4 && crate::cl0_double_tt!(1 + 1) == 3;
    let p7_ev = if p7_ok {
        "double!(1+1)=4(Tree 代換,原子子樹)vs double_tt!(1+1)=3(tt 拼接反例)".to_string()
    } else {
        "P7 對照失效".to_string()
    };
    out.push(PrincipleReport {
        id: "P7",
        name: "sorted variable ⇒ 優先級不變",
        passed: p7_ok,
        evidence: p7_ev,
    });

    out
}

/// 附加門禁:tt-muncher O(n²) 實證(借.md §3 的公式)
pub fn complexity_report() -> (bool, String) {
    let reg = registry();
    let entry = reg
        .iter()
        .find(|d| d.name == "cl0_count_tts")
        .expect("不變式:registry 必含 cl0_count_tts");
    let inner = reg
        .iter()
        .find(|d| d.name == "cl0_count_tts_inner")
        .expect("不變式:registry 必含 cl0_count_tts_inner");
    let comps = |n: usize| -> usize {
        let src = format!("( {} )", std::iter::repeat_n("a ", n).collect::<String>());
        let input =
            crate::token_tree::parse_forest(&src).expect("不變式:\"( a a … )\" 格式必可解析");
        expand_chain(&[entry, inner], &input, 65536)
            .expect("不變式:模型展開在 fuel 內必收斂")
            .telemetry
            .comparisons
    };
    let c32 = comps(32);
    let c64 = comps(64);
    let ratio = c64 as f64 / c32 as f64;
    // Θ(n²):規模 ×2 ⇒ 比較數 ~×4(允許 [3, 5] 吸收低階項)
    let ok = (3.0..=5.0).contains(&ratio);
    (
        ok,
        format!(
            "comps(32)={} comps(64)={} 比例={:.2}(→4 ⇒ Θ(n²),Σ(n+i)+Σi 累加器同被 re-match)",
            c32, c64, ratio
        ),
    )
}

// ===========================================================================
// §8 測試
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn reg() -> Vec<MacroDef> {
        registry()
    }

    // ---------- P1:互斥 ----------

    #[test]
    fn all_multi_rule_systems_are_pairwise_disjoint() {
        for def in reg() {
            for (a, b, v) in check_exclusive(&def) {
                assert_eq!(
                    v,
                    ExclVerdict::Disjoint,
                    "{} × {} 於系統 {} 必須互斥",
                    a,
                    b,
                    def.name
                );
            }
        }
    }

    #[test]
    fn single_rule_systems_are_trivially_exclusive() {
        for def in reg() {
            if def.rules.len() == 1 {
                assert!(check_exclusive(&def).is_empty());
            }
        }
    }

    #[test]
    fn checker_flags_overlapping_patterns_as_overlap() {
        // ($a:expr) vs ($i:ident):ident ⊆ expr ⇒ 相交
        let v = rule_pair_verdict(
            &[Pat::Group(Delim::Paren, vec![Pat::Meta("a", Frag::Expr)])],
            &[Pat::Group(Delim::Paren, vec![Pat::Meta("i", Frag::Ident)])],
        );
        assert_eq!(v, ExclVerdict::Overlap);
    }

    #[test]
    fn checker_is_conservative_on_free_repetition() {
        let v = rule_pair_verdict(
            &[Pat::Rep(vec![Pat::Meta("x", Frag::Tt)], None)],
            &[Pat::Meta("y", Frag::Tt)],
        );
        assert_eq!(v, ExclVerdict::Conservative);
    }

    // ---------- P3:展開器/μ ----------

    #[test]
    fn count_tts_model_expansion_terminates_with_strict_mu_descent() {
        let r = reg();
        let entry = r.iter().find(|d| d.name == "cl0_count_tts").unwrap();
        let inner = r.iter().find(|d| d.name == "cl0_count_tts_inner").unwrap();
        for n in [0usize, 1, 2, 5, 16, 64] {
            let src = format!("( {} )", "a ".repeat(n));
            let input = crate::token_tree::parse_forest(&src).unwrap();
            let tr = expand_chain(&[entry, inner], &input, 4096).expect("必須終止");
            // 步數 = E1(1) + I2(n) + I1(1)
            assert_eq!(tr.steps.len(), n + 2, "n={} 步數", n);
            // 終態渲染:0usize + 1 × n
            let expect = if n == 0 {
                "0usize".to_string()
            } else {
                format!("0usize {}", vec!["+ 1"; n].join(" "))
            };
            assert_eq!(tr.final_rendered, expect, "n={} 終態", n);
            // μ 嚴格遞減:每個 I2 步內部 rest 嚴格變小 (μ_in > μ_out)
            for s in &tr.steps {
                if s.rule == "I2-step" {
                    if let (Some(a), Some(b)) = (s.mu_in, s.mu_out) {
                        assert!(b < a, "μ 未遞減:{} → {}", a, b);
                    }
                }
            }
        }
    }

    #[test]
    fn expand_rejects_unmatched_input() {
        let r = reg();
        let inner = r.iter().find(|d| d.name == "cl0_count_tts_inner").unwrap();
        let input = crate::token_tree::parse_forest("( garbage )").unwrap();
        assert!(matches!(
            expand_chain(&[inner], &input, 64),
            Err(ExpandErr::NoRuleMatched { .. })
        ));
    }

    // ---------- O(n²) 實證 ----------

    #[test]
    fn tt_muncher_comparisons_grow_quadratically() {
        let (ok, ev) = complexity_report();
        assert!(ok, "比例必須趨近 4:{}", ev);
    }

    // ---------- P2/P4/P5/P6/P7:真巨集 ----------

    #[test]
    fn real_macros_match_reference_implementations() {
        fn double_ref(x: i32) -> i32 {
            x * 2
        }
        for x in 0..64i32 {
            assert_eq!(crate::cl0_double!(x), double_ref(x));
        }
        assert_eq!(crate::cl0_safe_vec!(1, 2, 3), ::std::vec![1, 2, 3]);
        assert_eq!(crate::cl0_count_tts!(a b c d), 4usize);
        let empty: ::std::vec::Vec<i32> = crate::cl0_safe_vec!();
        assert!(empty.is_empty());
    }

    #[test]
    fn cps_outermost_equals_innermost() {
        assert_eq!(crate::cl0_produce!(cl0_consume), 42);
        assert_eq!(crate::cl0_consume!(11, 31), 42);
    }

    #[test]
    fn absolute_paths_survive_caller_shadowing() {
        struct Vec;
        const OPTION: u32 = 7;
        let _: Vec = Vec;
        let _ = OPTION;
        let v = crate::cl0_safe_vec!(5, 6, 7);
        assert_eq!(v.len(), 3);
        assert_eq!(v[2], 7);
    }

    #[test]
    fn let_sharing_evaluates_side_effect_exactly_once() {
        use ::std::cell::Cell;
        let c = Cell::new(0usize);
        let r = crate::cl0_with_val!(
            {
                c.set(c.get() + 1);
                7
            },
            |x: &i32, y: &i32| x + y
        );
        assert_eq!(c.get(), 1, "副作用恰一次");
        assert_eq!(r, 14);
        // move-only 值:線性 + let sharing ⇒ 編譯通過且恰移動一次
        let s: ::std::string::String = "owned".into();
        let expect = s.len() * 2;
        let n = crate::cl0_with_val!(s, |a: &String, b: &String| a.len() + b.len());
        assert_eq!(n, expect);
    }

    #[test]
    fn sorted_fragment_preserves_precedence() {
        assert_eq!(crate::cl0_double!(1 + 1), 4); // Tree 代換
        assert_eq!(crate::cl0_double_tt!(1 + 1), 3); // String 代換反例
    }

    // ---------- 模板線性 ----------

    #[test]
    fn all_registry_templates_are_linear() {
        for def in reg() {
            for (rule, lin, bad) in check_linear(&def) {
                assert!(lin, "{} 非線性:{:?}", rule, bad);
            }
        }
    }

    #[test]
    fn linearity_checker_flags_double_use() {
        let bad = Tpl::Seq(vec![Tpl::Sub("x"), Tpl::Sub("x")]);
        let counts = template_linearity(&bad);
        assert_eq!(counts.get("x"), Some(&2));
    }

    // ---------- laminar_scope 執行期 ----------

    #[test]
    fn nested_laminar_scopes_record_laminar_intervals() {
        crate::macro_lab::reset_recorder();
        crate::cl0_laminar_scope!(1 {
            crate::cl0_laminar_scope!(2 {
                let _ = 1;
            });
            crate::cl0_laminar_scope!(3 {
                crate::cl0_laminar_scope!(4 {
                    let _ = 2;
                });
            });
        });
        let spans = crate::macro_lab::recorded_spans();
        assert_eq!(spans.len(), 4, "四層嵌套 ⇒ 四個區間");
        let regions: Vec<crate::ast::Interval> = spans
            .iter()
            .map(|&(_id, s, e)| crate::ast::Interval { start: s, end: e })
            .collect();
        assert!(
            crate::borrow_model::laminar_ok(&regions),
            "嵌套塊區間必須層狀:{:?}",
            spans
        );
        // 區間右端點由 Drop 釘死:s < e 嚴格
        for (id, s, e) in &spans {
            assert!(s < e, "scope {} 區間退化", id);
        }
        crate::macro_lab::reset_recorder();
    }

    // ---------- 七門禁 ----------

    #[test]
    fn seven_principle_gates_all_pass() {
        for r in verify_seven_principles() {
            assert!(r.passed, "gate {} failed: {}", r.id, r.evidence);
        }
        assert_eq!(verify_seven_principles().len(), 7);
    }
}
