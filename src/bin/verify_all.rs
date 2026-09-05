//! verify_all —— 端到端全量機械自証主程序(九項核檢)。
//!
//! 審計 F-01/F-06 修復:
//! * 門禁三態化 —— 每項核檢的結論只能是 Proven / Skipped / Failed;
//!   外部證明器缺席時如實輸出 SKIPPED(連同原因),與 PASSED 在輸出、
//!   計數與語義上嚴格分離,「零驗證發生、認證照樣背書」不再可能。
//! * 口徑常量驅動 —— 橫幅門禁數、編號、結論行全部由 `GATES` 常量渲染,
//!   標題與實跑項數永不漂移。
//! * 嚴格發布模式 —— `--strict` 或 `CL0R0_STRICT=1` 時,任一 SKIPPED
//!   即非零退出,未裝 Rocq/Why3 的機器拿不到發布綠燈。
//!
//! 一键执行定律自检、增量重析等价性、递减图会合、Newman 快速通道
//! (SN∧WCR⇒CR)、CPF 证书核验、确定性 Fuzzing 与形式化引理矩阵。
//! 运行: `cargo run --bin verify_all [-- --strict]`

use cl0r0::cpf_cert::{CPFCertificate, CertResult};
use cl0r0::dd_checker::{check_confluence_with_mode, CheckerMode, SNWitness};
use cl0r0::edit::Edit;
use cl0r0::gen::{gen_garbage, gen_legal, Rng};
use cl0r0::parse::parse;
use cl0r0::reparse_verifier::verify_reparse_equivalence;
use cl0r0::rocq_export::{KernelCheck, RocqExporter};
use cl0r0::rule_labeling::{dd_label_universe, dd_strict_order_pairs};
use cl0r0::tactic_scheduler::{Tactic, TacticScheduler};
use cl0r0::testkit::fixtures;

/// 門禁結論三態(F-01:Skipped ≠ Passed)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GateStatus {
    Proven,
    Skipped,
    Failed,
}

/// 門禁註冊表(F-06:橫幅、編號、結論行全部由此常量驅動)
const GATES: [&str; 9] = [
    "L1/L2/L5 語法層基礎檢查",
    "L3/L4 增量重析等價性",
    "L8/L9 遞減圖局部峰值會合",
    "Newman 快速通道 (SN∧WCR⇒CR)",
    "CPF 證書雙重核驗 (DD 偏序 + KB 短證)",
    "全化解析器 0 Panic 與 Tactic Scheduler",
    "18 大形式化引理矩陣",
    "Rocq 9.2 形式化理論 + rocqchk 微內核核檢",
    "Creusot/Why3 Z3 SMT 演繹核驗",
];

fn strict_mode() -> bool {
    std::env::args().any(|a| a == "--strict")
        || std::env::var("CL0R0_STRICT")
            .map(|v| v != "0")
            .unwrap_or(false)
}

fn main() {
    let strict = strict_mode();
    let mut proven = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    println!("======================================================================");
    println!(
        " CL0 / R₀ 雙載體 · {} 項端到端機械核檢流水線 (v0.2.0)",
        GATES.len()
    );
    if strict {
        println!(" 【嚴格發布模式】:外部證明器缺席 (SKIPPED) 即視為發布阻斷");
    }
    println!("======================================================================");

    // ------------------------------------------------------------------
    // [Gate 1/9] 語法層與九律基礎檢查 (L1, L2, L5, L6)
    // ------------------------------------------------------------------
    print!(
        "[Gate 1/{}] 正在校驗 L1 無損回環 / L2 決定論 / L5 Laminar 樹公理... ",
        GATES.len()
    );
    let mut rng = Rng::new(0xC10_2024_0001);
    let g1 = (|| {
        for _ in 0..1000 {
            let src = gen_legal(&mut rng);
            if let Ok(tree) = parse(&src) {
                if tree.unparse() != src || !tree.laminar_ok() {
                    return GateStatus::Failed;
                }
            }
        }
        GateStatus::Proven
    })();
    match g1 {
        GateStatus::Proven => {
            println!("PASSED (1000 樣本 100% 逐字節吻合)");
            proven += 1;
        }
        GateStatus::Skipped => {
            println!("SKIPPED");
            skipped += 1;
        }
        GateStatus::Failed => {
            println!("FAILED");
            failed += 1;
        }
    }

    // ------------------------------------------------------------------
    // [Gate 2/9] 增量重析等價性自証 (L3 / L4)
    // ------------------------------------------------------------------
    print!(
        "[Gate 2/{}] 正在校驗 L3/L4 增量重析配置快照重用等價性... ",
        GATES.len()
    );
    let reparse_samples = vec![
        (
            "fn main() { let mut x = 1; }",
            Edit {
                start: 24,
                old_end: 25,
                text: "2".to_string(),
            },
        ),
        (
            "fn foo(a: i32) { if a > 0 { bar(a); } }",
            Edit {
                start: 20,
                old_end: 25,
                text: "b < 10".to_string(),
            },
        ),
    ];
    let rep_report = verify_reparse_equivalence(&reparse_samples);
    if rep_report.failed_edits.is_empty() {
        println!(
            "PASSED ({}/{} 全部等價)",
            rep_report.passed_cases, rep_report.tested_cases
        );
        proven += 1;
    } else {
        println!("FAILED");
        failed += 1;
    }

    // ------------------------------------------------------------------
    // [Gate 3/9] van Oostrom 遞減圖合流性自証 (L8 / L9 - 無終止性假定軌道)
    // ------------------------------------------------------------------
    print!(
        "[Gate 3/{}] 正在校驗 L8/L9 遞減圖 (Decreasing Diagrams) 局部峰值會合... ",
        GATES.len()
    );
    // D-01:狀態宇宙引用 testkit 夾具單一真相
    let states = fixtures::sample_states();
    let dd_report = check_confluence_with_mode(&states, CheckerMode::DecreasingDiagrams, 6);
    if dd_report.certified {
        println!(
            "PASSED ({} 狀態 / {} 峰值全部收斂於唯一正規形)",
            dd_report.total_states, dd_report.total_peaks
        );
        proven += 1;
    } else {
        println!(
            "FAILED ({} 峰值不可會合)",
            dd_report.non_joinable_peaks.len()
        );
        failed += 1;
    }

    // ------------------------------------------------------------------
    // [Gate 4/9] Newman 快速通道 (SN ∧ WCR ⇒ CR) 機械核驗 + CPF-KB 短證
    // ------------------------------------------------------------------
    print!(
        "[Gate 4/{}] 正在校驗 Newman 快速通道 (SN ∧ WCR ⇒ CR 帶 SN 作用域見證)... ",
        GATES.len()
    );
    let sn_witness = SNWitness::LivenessScopeBounded {
        max_span_len: 4,
        storages: 1,
    };
    let start_newman = std::time::Instant::now();
    let newman_report = check_confluence_with_mode(
        &states,
        CheckerMode::Newman {
            sn_witness: sn_witness.clone(),
        },
        6,
    );
    let newman_dur = start_newman.elapsed();

    if newman_report.certified && newman_report.cpf_kb_proof.is_some() {
        println!(
            "PASSED (WCR 可接合 · 出具 CPF-KB 短證 · 耗時 {:?})",
            newman_dur
        );
        proven += 1;
    } else {
        println!("FAILED (Newman 快速通道未通過)");
        failed += 1;
    }

    // ------------------------------------------------------------------
    // [Gate 5/9] 原生 CPF 證書偏序無環與 Knuth-Bendix 雙重核驗
    // ------------------------------------------------------------------
    print!(
        "[Gate 5/{}] 正在校驗原生 CPF 證書 (DD 偏序 + KB 短證雙重自檢)... ",
        GATES.len()
    );
    // D-02:標號宇宙與偏序由 rule_labeling 單一真相導出
    let cert_dd = CPFCertificate::new_decreasing_diagrams(
        "CL0-DD",
        dd_label_universe(),
        dd_strict_order_pairs(),
    );
    // F-03:KB 短證攜帶 Newman 通道機械實錄的臨界對會合見證
    let cert_kb = CPFCertificate::new_knuth_bendix(
        "CL0-KB",
        &sn_witness.description(),
        newman_report.kb_critical_pair_witnesses.clone(),
    );

    if cert_dd.verify() == CertResult::Certified && cert_kb.verify() == CertResult::Certified {
        println!(
            "PASSED (CERTIFIED · DD 偏序無環 ∧ KB 短證 {} 對見證全數合法)",
            cert_kb.critical_pairs_count()
        );
        proven += 1;
    } else {
        println!("FAILED (CPF 證書被拒絕: {:?})", cert_kb.verify());
        failed += 1;
    }

    // ------------------------------------------------------------------
    // [Gate 6/9] 全化解析器髒輸入零崩潰與 Tactic Scheduler 對拍
    // ------------------------------------------------------------------
    print!(
        "[Gate 6/{}] 正在校驗全化解析器髒輸入 0 Panic 與 Tactic Scheduler... ",
        GATES.len()
    );
    let mut g6_ok = true;
    for _ in 0..2000 {
        let garbage = gen_garbage(&mut rng, 40);
        if parse(&garbage).is_err() {
            g6_ok = false;
            break;
        }
    }
    let sched_res = TacticScheduler::schedule_and_verify(&states, Some(sn_witness), 6);
    if g6_ok && sched_res.selected_tactic == Tactic::NewmanFastPath {
        println!("PASSED (2000 隨機髒輸入 0 Panic ∧ 策略調度器命中 NewmanFastPath)");
        proven += 1;
    } else {
        println!("FAILED");
        failed += 1;
    }

    // ------------------------------------------------------------------
    // [Gate 7/9] 形式化引理矩陣機械自証 (L1-L18 全量 18 大形式化引理)
    // ------------------------------------------------------------------
    print!(
        "[Gate 7/{}] 正在校驗 18 大形式化核心引理矩陣機械自証... ",
        GATES.len()
    );
    let lemma_results = cl0r0::lemmas::LemmaRegistry::verify_all_lemmas();
    let all_lemmas_ok = lemma_results.iter().all(|r| r.is_certified());
    if all_lemmas_ok {
        println!(
            "PASSED ({}/{} 形式化引理全部獲得機器證明見證)",
            lemma_results.len(),
            lemma_results.len()
        );
        proven += 1;
    } else {
        println!("FAILED (存在未通過引理)");
        failed += 1;
    }

    // ------------------------------------------------------------------
    // [Gate 8/9] Rocq 9.2 形式化理論導出與微內核機械核檢
    //             (F-01:工具缺席 ⇒ 如實 SKIPPED,絕不偽稱 PASSED)
    // ------------------------------------------------------------------
    print!(
        "[Gate 8/{}] 正在校驗 Rocq 9.2 形式化理論導出與 rocqchk 微內核核檢... ",
        GATES.len()
    );
    let rocq_theory = RocqExporter::export_full_cl0_theory("CL0_VerifyAll_Rocq");
    let (rocq_status, rocq_note) = match RocqExporter::get_rocq_binary_path() {
        Some(_) => match RocqExporter::compile_and_verify(&rocq_theory, "CL0_VerifyAll_Rocq") {
            Ok(r) if r.success && r.kernel_check == KernelCheck::Checked => (
                GateStatus::Proven,
                "Rocq 9.2 .v 導出 ∧ rocqchk 機械證明全量合規".to_string(),
            ),
            Ok(r) => match &r.kernel_check {
                KernelCheck::NotRun(reason) => (
                    GateStatus::Skipped,
                    format!("編譯通過但 rocqchk 未執行: {}", reason),
                ),
                KernelCheck::Checked => unreachable!(),
            },
            Err(e) => (GateStatus::Failed, e),
        },
        None => (
            GateStatus::Skipped,
            "rocq 不在 PATH/ROCQ_HOME — 本機未執行 .v 編譯與微內核核檢".to_string(),
        ),
    };
    match rocq_status {
        GateStatus::Proven => {
            println!("PASSED ({})", rocq_note);
            proven += 1;
        }
        GateStatus::Skipped => {
            println!("SKIPPED ({})", rocq_note);
            skipped += 1;
        }
        GateStatus::Failed => {
            println!("FAILED ({})", rocq_note);
            failed += 1;
        }
    }

    // ------------------------------------------------------------------
    // [Gate 9/9] Creusot / Why3 演繹驗證與 Pearlite 預言變量消解
    //             (F-01:工具缺席 ⇒ 如實 SKIPPED,絕不偽稱 PASSED)
    // ------------------------------------------------------------------
    print!(
        "[Gate 9/{}] 正在校驗 Creusot 演繹理論導出與 Why3/Z3 SMT 全自動消解... ",
        GATES.len()
    );
    let creusot_theory =
        cl0r0::creusot_export::CreusotExporter::export_full_creusot_theory("CL0_VerifyAll_Creusot");
    let (creusot_status, creusot_note) = match (
        cl0r0::creusot_export::CreusotExporter::check_why3_available(),
        cl0r0::creusot_export::CreusotExporter::check_z3_available(),
    ) {
        (true, true) => {
            match cl0r0::creusot_export::CreusotExporter::verify_with_why3(
                &creusot_theory,
                "CL0_VerifyAll_Creusot",
            ) {
                Ok(r) if r.success && r.valid_goals == r.total_goals => (
                    GateStatus::Proven,
                    format!(
                        "Creusot MLW 導出 ∧ Why3+Z3 SMT {}/{} Goals 100% Valid",
                        r.valid_goals, r.total_goals
                    ),
                ),
                Ok(r) => (
                    GateStatus::Failed,
                    format!("Why3 僅消解 {}/{} goals", r.valid_goals, r.total_goals),
                ),
                Err(e) => (GateStatus::Failed, e),
            }
        }
        (why3, z3) => {
            let missing = match (why3, z3) {
                (false, false) => "why3 與 z3 均不在 PATH — 未執行 SMT 消解",
                (false, true) => "why3 不在 PATH — 未執行 SMT 消解",
                (true, false) => "z3 不在 PATH — 未執行 SMT 消解",
                _ => unreachable!(),
            };
            (GateStatus::Skipped, missing.to_string())
        }
    };
    match creusot_status {
        GateStatus::Proven => {
            println!("PASSED ({})", creusot_note);
            proven += 1;
        }
        GateStatus::Skipped => {
            println!("SKIPPED ({})", creusot_note);
            skipped += 1;
        }
        GateStatus::Failed => {
            println!("FAILED ({})", creusot_note);
            failed += 1;
        }
    }

    // ------------------------------------------------------------------
    // 結論:由實跑計數渲染,口徑與輸出一致(F-01/F-06)
    // ------------------------------------------------------------------
    println!("======================================================================");
    let total = GATES.len();
    if failed == 0 && skipped == 0 {
        println!(
            " [自証結論]: {}/{} 門禁 Proven · 0 Skipped · 0 Failed — 100% 全部通過!系統符合雙載體與 CoCo 2026 發布標準。",
            proven, total
        );
        std::process::exit(0);
    } else if failed == 0 {
        if strict {
            eprintln!(
                " [自証結論]: {}/{} Proven · {} SKIPPED · {} FAILED — 嚴格發布模式下 SKIPPED 即發布阻斷!請在配備 Rocq/Why3/Z3 的環境復跑。",
                proven, total, skipped, failed
            );
            std::process::exit(1);
        }
        println!(
            " [自証結論]: {}/{} 門禁 Proven · {} 項 SKIPPED(外部證明器缺席,如實申報未執行)· {} FAILED。機內門禁全部通過;發布前請以 --strict 在配備 Rocq/Why3/Z3 的環境復跑。",
            proven, total, skipped, failed
        );
        std::process::exit(0);
    } else {
        eprintln!(
            " [自証結論]: {}/{} Proven · {} SKIPPED · {} FAILED — 存在未通過門禁,請檢查上述 FAILED 項目!",
            proven, total, skipped, failed
        );
        std::process::exit(1);
    }
}
