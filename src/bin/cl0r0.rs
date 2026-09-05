//! cl0r0 —— 演示程序:CL0 載體上的九律檢查 + 幾何/重寫演示 + R₀ 接線預備。
//!
//! DL-008:樹公理檢查、L7b 迭代淨化、修剪計劃已下沉 `cl0r0::fuzz_engine`
//! (可單測);本 bin 只剩演示渲染。
//!
//! 運行:`cargo run --bin cl0r0`(全綠輸出 0 個失敗即為機械自証通過)。

use cl0r0::ast;
use cl0r0::fuzz_engine::{
    iterative_purify, normalize_plan, residual_bad_errors, tree_axiom_checks,
};
use cl0r0::parse::parse;
use cl0r0::r0;

fn ok(b: bool) -> &'static str {
    if b {
        "✓"
    } else {
        "✗ FAIL"
    }
}

fn main() {
    println!("======================================================================");
    println!(" CL0 / R₀ 雙載體 · 機械自証演示(九條定律 + 幾何 + 重寫系統)");
    println!("======================================================================");

    // ---------- 樣本:一個含借用衝突的 CL0 程式 ----------
    let src = "fn main() {\n\
               \x20 let mut x = 1;\n\
               \x20 let r = &mut x;\n\
               \x20 let y = x + 1;\n\
               \x20 let z = *r;\n\
               \x20 while z < 10 { f(y); }\n\
               \x20 if z == 3 { f(x); } else { g(); }\n\
               }";
    println!("\n[演示輸入]\n{}", src);

    // ---------- L1/L2/L5 樹性質(引擎下沉:fuzz_engine) ----------
    let t = parse(src).expect("全化解析器:任何輸入都產出樹");
    let (named, anon, errs, trivia) = t.stats();
    println!(
        "\n[L1/L2/L5 樹性質] 節點 {} (具名 {} 匿名 {} trivia {} error {})",
        t.total_nodes(),
        named,
        anon,
        trivia,
        errs
    );
    let ax = tree_axiom_checks(src);
    println!(
        "  L1 無損回環(unparse∘parse ≡ 源碼逐字節):{}",
        ok(ax.roundtrip)
    );
    println!("  L2 決定論(兩次解析序列化相同):{}", ok(ax.determinism));
    println!("  L5 區間嵌套(laminar):{}", ok(ax.laminar));
    println!("  連續性公理(父跨度 = 子跨度並):{}", ok(ax.continuity));
    println!("  樹公理(連通/單父):{}", ok(ax.tree_shapes));
    println!("  χ = |V| − |E| = 1(CW 複形,§3.4):{}", ok(ax.euler_is_one));

    // ---------- 具名投影 ----------
    println!("\n[具名投影 §1.3](匿名 token 與 trivia 投影掉;結構同態)");
    println!("  {}", t.named_sexp());

    // ---------- 三軌 liveness + 衝突圖 ----------
    let facts = ast::extract(&t);
    println!(
        "\n[事實層 §3.2] 綁定 {} 事件 {} 借用鏈 {}",
        facts.bindings.len(),
        facts.events.len(),
        facts.links.len()
    );
    for line in cl0r0::fuzz_engine::facts_summary_lines(&facts) {
        println!("  {}", line);
    }

    let mut total_red = 0usize;
    for ts in cl0r0::fuzz_engine::track_summaries(&facts) {
        total_red += ts.edge_lines.len();
        println!("\n[{} 軌] 紅邊 {} 條:", ts.label, ts.edge_lines.len());
        for line in &ts.edge_lines {
            println!("  {}", line);
        }
        println!(
            "  [定理 T2 實例] 區間圖 ω = {} χ = {} → χ = ω(完美圖,弦圖):{}",
            ts.omega,
            ts.chi,
            ok(ts.omega == ts.chi)
        );
    }

    // ---------- 幾何演示:把紅邊清零的「修剪」計劃(引擎下沉) ----------
    println!("\n[修法菜單 §4] 以 referent 軌的紅邊為對象,運行規範修剪菜單");
    let plan = normalize_plan(src);
    println!(
        "  初始:紅邊 {} 條;正規化 {} 步;最終紅邊 {} 條",
        plan.initial_red, plan.steps, plan.final_red
    );
    for (label, before, after) in &plan.plan {
        println!("    {} : 紅邊 {} → {}", label, before, after);
    }
    println!(
        "  結果(紅邊清零 = 幾何收斂,§3.5):{}",
        ok(plan.final_red == 0)
    );

    // ---------- ERROR 全化(L7)演示(引擎下沉) ----------
    println!("\n[L7 ERROR 全化 §2.3]「寫到一半的檔案」");
    for half in [
        "fn main() {\n  let mut x = 1;\n  let r = &mut",
        "fn main() {\n  let y = x +",
        "fn main() { if x { } else {",
        "@@@",
    ] {
        let h = parse(half).expect("total");
        let spans = h.maximal_error_spans();
        let out = iterative_purify(half);
        let residual = residual_bad_errors(&out.purified, &out.seams);
        println!(
            "  {:?}\n    → ERROR 跨度 {} 個({:?});迭代挖除 {} 輪後殘餘 {} 錯誤 → L7b 良構極大:{}",
            half,
            spans.len(),
            spans,
            out.rounds,
            residual,
            ok(residual == 0)
        );
    }

    // ---------- R₀ 接線預備 ----------
    println!("\n[R₀ 附錄 B 接線] LALR(1)-乾淨片段 + unsupported 如實申報");
    println!("  EBNF:\n{}", r0::R0_EBNF);
    let r0src = "fn main() {\n  let mut x = 1;\n  let r = &mut x;\n  *r = *r + 1;\n  loop { if x < 42 { return; } }\n}";
    println!("  片段:\n{}", r0src);
    println!(
        "  r0 詞法平鋪:{};  LALR(1)-乾淨:{}",
        ok(r0::r0_lexical_invariants(r0src).is_ok()),
        ok(r0::lalr1_clean(r0src).is_ok())
    );
    let outof =
        "fn main() { let c = |a| a; match c { 0 => {} } let v: Vec<int> = vs; println!(x); }";
    println!("  越界樣本:{:?}", outof);
    for (k, sp) in &r0::unsupported(outof) {
        println!("    unsupported: {} @{}", k, sp);
    }
    println!(
        "  lalr1_clean 越界樣本(應為 Err):{}",
        ok(r0::lalr1_clean(outof).is_err())
    );

    println!("\n======================================================================");
    println!(" 總計:九律矩陣的機械檢查結果見 `tests/laws.rs` 與 `cargo run --bin l9newman`");
    println!("======================================================================");
    let _ = total_red;
}
