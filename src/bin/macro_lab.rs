//! macro_lab —— 巨集七原則 + 借用組合模型 · 證據鏈主程序。
//!
//! 對應 MACRO_SEVEN_PRINCIPLES.md 佈署計畫的驗收層:
//! * P1–P7 七門禁(規則互斥、模型↔真巨集同構、μ 良基遞減、
//!   組合語義、hygiene、let 共享、Tree 代換);
//! * B1–B6 借用組合門禁(κ-矩陣 3 衝突格、naive≡sweep≡Datalog、
//!   層狀棧比較數優勢、3·o²·n² 會計、rep_dd 紅邊交叉驗證);
//! * 3·o²·n² 複雜度實測(comps(64)/comps(32) ∈ [3,5] ⇒ Θ(n²))。
//!
//! 運行: `cargo run --bin macro_lab [-- --verbose]`

use cl0r0::borrow_model::verify_borrow_model;
use cl0r0::macro_lab::{complexity_report, registry, verify_seven_principles};

fn main() {
    let verbose = std::env::args().any(|a| a == "--verbose" || a == "-v");
    let mut failed = 0usize;

    println!("======================================================================");
    println!(" 巨集七原則 × 借用組合模型 · 證據鏈流水線 (macro_lab)");
    println!("======================================================================");

    // ------------------------------------------------------------------
    // 第一段:P1–P7 七門禁
    // ------------------------------------------------------------------
    println!();
    println!("[I] 巨集七原則(verify_seven_principles)");
    for r in verify_seven_principles() {
        let mark = if r.passed { "PASS" } else { "FAIL" };
        if !r.passed {
            failed += 1;
        }
        println!("  [{}] {} — {}", mark, r.id, r.name);
        if verbose || !r.passed {
            println!("        證據: {}", r.evidence);
        }
    }

    // ------------------------------------------------------------------
    // 第二段:B1–B6 借用組合模型門禁
    // ------------------------------------------------------------------
    println!();
    println!("[II] 借用組合模型(verify_borrow_model)");
    for r in verify_borrow_model() {
        let mark = if r.passed { "PASS" } else { "FAIL" };
        if !r.passed {
            failed += 1;
        }
        println!("  [{}] {} — {}", mark, r.id, r.name);
        if verbose || !r.passed {
            println!("        證據: {}", r.evidence);
        }
    }

    // ------------------------------------------------------------------
    // 第三段:3·o²·n² 複雜度實測
    // ------------------------------------------------------------------
    println!();
    println!("[III] 複雜度實測(complexity_report)");
    let (ok, ev) = complexity_report();
    if !ok {
        failed += 1;
    }
    println!("  [{}] Θ(n²) 比例檢驗", if ok { "PASS" } else { "FAIL" });
    println!("        證據: {}", ev);

    // ------------------------------------------------------------------
    // 附錄:registry 系統規則樹清單(模型↔真巨集同構登記)
    // ------------------------------------------------------------------
    println!();
    println!("[附錄] registry 系統規則樹({} 系統)", registry().len());
    for d in registry() {
        println!(
            "  · {:<22} 規則 {:>2} 條 · 原則 [{}]",
            d.name,
            d.rules.len(),
            d.principles.join(",")
        );
        if verbose {
            for r in &d.rules {
                println!(
                    "      - {:<14} lhs {} pat / rhs {:?}{}",
                    r.name,
                    r.lhs.len(),
                    r.rhs,
                    if r.delegate_after { " · 委派" } else { "" }
                );
            }
        }
    }

    // ------------------------------------------------------------------
    // 結論
    // ------------------------------------------------------------------
    println!("======================================================================");
    let total = 7 + 6 + 1;
    if failed == 0 {
        println!(
            " [自証結論]: {}/{} 門禁 PASS — 七原則證據鏈與借用組合模型全部成立。",
            total - failed,
            total
        );
        std::process::exit(0);
    } else {
        eprintln!(
            " [自証結論]: {}/{} 門禁 PASS — {} 項 FAILED,請檢查上述證據!",
            total - failed,
            total,
            failed
        );
        std::process::exit(1);
    }
}
