//! verify_all —— 端到端全量機械自証主程序(十項核檢)。
//!
//! 審計 F-01/F-06 修復(語義不變):
//! * 門禁三態化 —— Proven / Skipped / Failed;外部證明器缺席時如實輸出
//!   SKIPPED(連同原因),與 PASSED 嚴格分離;
//! * 口徑常量驅動 —— 橫幅門禁數、編號、結論行全部由 `GATES` 常量渲染;
//! * 嚴格發布模式 —— `--strict` / `CL0R0_STRICT=1` 時任一 SKIPPED 即非零退出。
//!
//! DL-004:門禁「檢查 + 判定」決策層已下沉 `cl0r0::selfcheck`(可單測);
//! 本 bin 只剩橫幅、編號與結論渲染。
//!
//! 运行: `cargo run --bin verify_all [-- --strict]`

use cl0r0::selfcheck::{self, GateLedger};

/// 門禁註冊表(F-06:橫幅、編號、結論行全部由此常量驅動)
const GATES: [&str; 10] = [
    "L1/L2/L5 語法層基礎檢查",
    "L3/L4 增量重析等價性",
    "L8/L9 遞減圖局部峰值會合",
    "Newman 快速通道 (SN∧WCR⇒CR)",
    "CPF 證書雙重核驗 (DD 偏序 + KB 短證)",
    "全化解析器 0 Panic 與 Tactic Scheduler",
    "18 大形式化引理矩陣",
    "Rocq 9.2 形式化理論 + rocqchk 微內核核檢",
    "Creusot/Why3 Z3 SMT 演繹核驗",
    "巨集七原則 P1–P7 + 借用組合 B1–B6 + Θ(n²) 實測",
];

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
    println!(
        " CL0 / R₀ 雙載體 · {} 項端到端機械核檢流水線 (v0.2.0)",
        GATES.len()
    );
    if strict {
        println!(" 【嚴格發布模式】:外部證明器缺席 (SKIPPED) 即視為發布阻斷");
    }
    println!("======================================================================");

    let n = GATES.len();
    let states = selfcheck::core_states();
    let sn = selfcheck::core_sn_witness();

    print!("[Gate 1/{n}] 正在校驗 L1 無損回環 / L2 決定論 / L5 Laminar 樹公理... ");
    ledger.record(selfcheck::gate_l1_l2_l5(1000, 0xC10_2024_0001));

    print!("[Gate 2/{n}] 正在校驗 L3/L4 增量重析配置快照重用等價性... ");
    ledger.record(selfcheck::gate_reparse_equiv());

    print!("[Gate 3/{n}] 正在校驗 L8/L9 遞減圖 (Decreasing Diagrams) 局部峰值會合... ");
    ledger.record(selfcheck::gate_dd_peaks(&states));

    print!("[Gate 4/{n}] 正在校驗 Newman 快速通道 (SN ∧ WCR ⇒ CR 帶 SN 作用域見證)... ");
    let (newman_outcome, newman_report) = selfcheck::gate_newman_fastpath(&states, &sn);
    ledger.record(newman_outcome);

    print!("[Gate 5/{n}] 正在校驗原生 CPF 證書 (DD 偏序 + KB 短證雙重自檢)... ");
    ledger.record(selfcheck::gate_cpf_dual(&newman_report, &sn));

    print!("[Gate 6/{n}] 正在校驗全化解析器髒輸入 0 Panic 與 Tactic Scheduler... ");
    ledger.record(selfcheck::gate_dirty_input_scheduler(
        2000,
        0xC10_2024_0001,
        &states,
        &sn,
    ));

    print!("[Gate 7/{n}] 正在校驗 18 大形式化核心引理矩陣機械自証... ");
    ledger.record(selfcheck::gate_lemma_matrix());

    print!("[Gate 8/{n}] 正在校驗 Rocq 9.2 形式化理論導出與 rocqchk 微內核核檢... ");
    ledger.record(selfcheck::gate_rocq("CL0_VerifyAll_Rocq"));

    print!("[Gate 9/{n}] 正在校驗 Creusot 演繹理論導出與 Why3/Z3 SMT 全自動消解... ");
    ledger.record(selfcheck::gate_creusot("CL0_VerifyAll_Creusot"));

    print!("[Gate 10/{n}] 正在校驗巨集七原則 P1–P7、借用組合 B1–B6 與 Θ(n²) 複雜度... ");
    ledger.record(selfcheck::gate_macro_seven());

    // ------------------------------------------------------------------
    // 結論:由實跑計數渲染,口徑與輸出一致(F-01/F-06)
    // ------------------------------------------------------------------
    println!("======================================================================");
    let total = GATES.len();
    if ledger.failed == 0 && ledger.skipped == 0 {
        println!(
            " [自証結論]: {}/{} 門禁 Proven · 0 Skipped · 0 Failed — 100% 全部通過!系統符合雙載體與 CoCo 2026 發布標準。",
            ledger.proven, total
        );
        std::process::exit(0);
    } else if ledger.failed == 0 {
        if strict {
            eprintln!(
                " [自証結論]: {}/{} Proven · {} SKIPPED · {} FAILED — 嚴格發布模式下 SKIPPED 即發布阻斷!請在配備 Rocq/Why3/Z3 的環境復跑。",
                ledger.proven, total, ledger.skipped, ledger.failed
            );
            std::process::exit(1);
        }
        println!(
            " [自証結論]: {}/{} 門禁 Proven · {} 項 SKIPPED(外部證明器缺席,如實申報未執行)· {} FAILED。機內門禁全部通過;發布前請以 --strict 在配備 Rocq/Why3/Z3 的環境復跑。",
            ledger.proven, total, ledger.skipped, ledger.failed
        );
        std::process::exit(0);
    } else {
        eprintln!(
            " [自証結論]: {}/{} Proven · {} SKIPPED · {} FAILED — 存在未通過門禁,請檢查上述 FAILED 項目!",
            ledger.proven, total, ledger.skipped, ledger.failed
        );
        std::process::exit(1);
    }
}
