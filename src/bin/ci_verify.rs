//! ci_verify —— 端到端 CI 門禁與全量形式化自証執行器(七項核檢)。
//!
//! 審計 F-01/F-07 語義(不變):
//! * Gate 2 進度文案由 `LemmaStressEvaluator::expected_total` 派生(與真值同源);
//! * Gate 5/6 內嵌 Rocq / Creusot 核檢三態化 —— 工具缺席 ⇒ 整項門禁如實
//!   SKIPPED(嚴格模式非零退出),絕不偽稱 PASSED;
//! * 結論行由實跑計數渲染,嚴格模式 `CL0R0_STRICT=1` 下 SKIPPED 即阻斷。
//!
//! DL-004:門禁「檢查 + 判定」決策層已下沉 `cl0r0::selfcheck`(可單測);
//! 本 bin 只剩橫幅、編號與結論渲染。
//!
//! 運行: `cargo run --bin ci_verify`

use cl0r0::selfcheck::{self, GateLedger};
use std::time::Instant;

fn strict_mode() -> bool {
    std::env::args().any(|a| a == "--strict")
        || std::env::var("CL0R0_STRICT")
            .map(|v| v != "0")
            .unwrap_or(false)
}

fn main() {
    let strict = strict_mode();
    let mut ledger = GateLedger::new(strict);
    println!("======================================================================");
    println!(" CL0 / R₀ 雙載體 · CI 全量自動化機械自証與門禁流水線 (CI Matrix Gate)");
    if strict {
        println!(" 【嚴格發布模式】:外部證明器缺席 (SKIPPED) 即視為發布阻斷");
    }
    println!("======================================================================");

    let start_all = Instant::now();

    print!("[CI Gate 1/7] 正在驗證 18 大形式化核心引理機器證明見證... ");
    ledger.record(selfcheck::ci_gate_lemma_matrix());

    // F-07:文案數字與真值同源(expected_total 派生)
    let (expected_samples, stress_outcome) = selfcheck::ci_gate_stress(0xC10_2024_0001, 5);
    print!(
        "[CI Gate 2/7] 正在執行 {} 組引理海量數據與高熵污料壓測... ",
        expected_samples
    );
    ledger.record(stress_outcome);

    print!("[CI Gate 3/7] 正在驗證五大組合深度合成閉環 (1+2+3+4+5)... ");
    ledger.record(selfcheck::ci_gate_synthesis());

    print!("[CI Gate 4/7] 正在校驗結構化 JSON 錯誤報告與自動修復消解... ");
    ledger.record(selfcheck::ci_gate_json_repair());

    print!("[CI Gate 5/7] 正在驗證 DAG 項指針共享、辨別樹索引與 Maude 引擎... ");
    ledger.record(selfcheck::ci_gate_dag_dt_unif_rocq());

    print!("[CI Gate 6/7] 正在驗證 MIR 控制流、OOPSLA 2025 契約、Aeneas/Creusot & Dropck/UCG... ");
    ledger.record(selfcheck::ci_gate_mir_contracts());

    print!("[CI Gate 7/7] 正在執行全維度差分審核與持久化結構共享驗證 (>= 92%)... ");
    ledger.record(selfcheck::ci_gate_differential_audit(50));

    let elapsed = start_all.elapsed();
    println!("======================================================================");
    if ledger.failed == 0 && ledger.skipped == 0 {
        println!(
            " [CI 最終結論]: 7/7 CI 門禁 Proven · 0 Skipped · 0 Failed — 100% 全部通過!總耗時: {:?}",
            elapsed
        );
        std::process::exit(0);
    } else if ledger.failed == 0 {
        if strict {
            eprintln!(
                " [CI 最終結論]: {}/7 Proven · {} SKIPPED · 0 FAILED — 嚴格發布模式下 SKIPPED 即發布阻斷!請在配備 Rocq/Why3/Z3 的環境復跑。總耗時: {:?}",
                ledger.proven, ledger.skipped, elapsed
            );
            std::process::exit(1);
        }
        println!(
            " [CI 最終結論]: {}/7 CI 門禁 Proven · {} 項 SKIPPED(外部證明器缺席,如實申報未執行)· 0 FAILED。機內門禁全部通過;發布前請以 CL0R0_STRICT=1 復跑。總耗時: {:?}",
            ledger.proven, ledger.skipped, elapsed
        );
        std::process::exit(0);
    } else {
        eprintln!(
            " [CI 最終結論]: {}/7 Proven · {} SKIPPED · {} FAILED — 存在失敗門禁,請檢查上述輸出!",
            ledger.proven, ledger.skipped, ledger.failed
        );
        std::process::exit(1);
    }
}
