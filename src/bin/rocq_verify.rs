//! rocq_verify —— Rocq 9.2 形式化證明導出與微內核機械核檢主程序。
//!
//! 審計 F-01/F-12 修復:
//! * rocq 缺席時如實輸出「未執行編譯與核檢」,結論行不再宣稱
//!   「100% 機械核檢通過」;
//! * rocqchk 三態回報(`KernelCheck`):Checked / NotRun(原因),
//!   同一布林不再一處當硬門禁、一處當裝飾;
//! * 嚴格模式(`--strict` / `CL0R0_STRICT=1`):rocq 或 rocqchk 缺席
//!   即非零退出。
//!
//! 運行: `cargo run --bin rocq_verify [-- --strict]`

use cl0r0::cpf_cert::{CPFCertificate, CriticalPairWitness};
use cl0r0::rocq_export::{KernelCheck, RocqExporter};
use std::time::Instant;

fn strict_mode() -> bool {
    std::env::args().any(|a| a == "--strict")
        || std::env::var("CL0R0_STRICT")
            .map(|v| v != "0")
            .unwrap_or(false)
}

fn main() {
    let strict = strict_mode();
    println!("======================================================================");
    println!(" Rocq 9.2 (The Rocq Prover) 形式化理論導出與微內核機械核檢引擎");
    if strict {
        println!(" 【嚴格發布模式】:rocq / rocqchk 缺席即非零退出");
    }
    println!("======================================================================");

    let rocq_available = RocqExporter::check_rocq_available();
    let mut kernel_check_performed = false;
    if !rocq_available {
        println!(" [如實申報]: 系統環境中未檢測到 Rocq 9.2 (The Rocq Prover) 可執行檔。");
        println!("             本輪【未執行】任何 .v 編譯與 rocqchk 微內核核檢,");
        println!("             僅執行形式化理論腳本 AST 合成與靜態校驗。");
    } else {
        println!(
            " [環境檢測]: 已檢測到 Rocq 9.2 可執行環境: {:?}",
            RocqExporter::get_rocq_binary_path()
        );
    }

    let start_all = Instant::now();

    // 1. 導出完整 CL0 / R0 雙載體基礎理論
    println!("\n>> [Stage 1/3] 正在合成 CL0/R₀ 雙載體 18 大形式化引理 Rocq 9.2 理論文檔...");
    let full_theory = RocqExporter::export_full_cl0_theory("CL0_Dual_Carrier_Rocq");
    println!("   - 理論文檔行數: {} 行", full_theory.lines().count());
    println!("   - 理論文檔大小: {} 字節", full_theory.len());

    // 2. 導出 Newman 與 Knuth-Bendix 證書理論
    //    (F-03:KB 證書由 Newman 通道機械實錄的臨界對會合見證構造)
    println!("\n>> [Stage 2/3] 正在合成 Newman 快速通道與 CPF-KB 短證 Rocq 9.2 理論文檔...");
    let kb_cert = CPFCertificate::new_knuth_bendix(
        "CL0_KB_System",
        "LivenessScopeBounded(span <= 4, storages = 1)",
        vec![CriticalPairWitness::new(
            "peak:trim(ev0)@1",
            "peak:runtime(0,1)",
            "nf:ev0[0,1)xev1[2,4)",
        )],
    );
    let cert_theory = RocqExporter::export_theory("CL0_KB_Certificate_Rocq", &kb_cert);

    // 3. 執行 Rocq 9.2 機械編譯與 rocqchk 雙重核驗
    println!("\n>> [Stage 3/3] 正在調用 Rocq 9.2 編譯器與微內核核檢器進行機械證明檢驗...");
    if rocq_available {
        let res1 = RocqExporter::compile_and_verify(&full_theory, "CL0_Dual_Carrier_Rocq");
        match res1 {
            Ok(rep) => {
                println!(
                    "   [Theory 1]: CL0_Dual_Carrier_Rocq.v -> .vo ({} 字節) 編譯通過 (耗時: {} ms)",
                    rep.vo_bytes, rep.compilation_duration_ms
                );
                match rep.kernel_check {
                    KernelCheck::Checked => {
                        kernel_check_performed = true;
                        println!("               rocqchk 微內核獨立核驗: PASSED (Modules were successfully checked)");
                    }
                    KernelCheck::NotRun(reason) => {
                        println!(
                            "               rocqchk 微內核獨立核驗: NOT RUN ({})",
                            reason
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("   [Theory 1 FAILED]: {}", e);
                std::process::exit(1);
            }
        }

        let res2 = RocqExporter::compile_and_verify(&cert_theory, "CL0_KB_Certificate_Rocq");
        match res2 {
            Ok(rep) => {
                println!(
                    "   [Theory 2]: CL0_KB_Certificate_Rocq.v -> .vo ({} 字節) 編譯通過 (耗時: {} ms)",
                    rep.vo_bytes, rep.compilation_duration_ms
                );
                match rep.kernel_check {
                    KernelCheck::Checked => {
                        kernel_check_performed = true;
                        println!("               rocqchk 微內核獨立核驗: PASSED (Modules were successfully checked)");
                    }
                    KernelCheck::NotRun(reason) => {
                        println!(
                            "               rocqchk 微內核獨立核驗: NOT RUN ({})",
                            reason
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("   [Theory 2 FAILED]: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("   [跳過編譯]: rocq 缺席 — 本輪未編譯任何 .v 文件,未執行任何機械核檢。");
    }

    let elapsed = start_all.elapsed();
    println!("\n======================================================================");
    if rocq_available && kernel_check_performed {
        println!(
            " [Rocq 9.2 驗證結論]: 理論編譯通過 ∧ rocqchk 微內核核檢 PASSED。耗時: {:?}",
            elapsed
        );
        std::process::exit(0);
    } else if rocq_available {
        // rocq 在,但 rocqchk 從未執行 —— 如實申報,嚴格模式阻斷
        if strict {
            eprintln!(
                " [Rocq 9.2 驗證結論]: 編譯通過,但 rocqchk 未執行 — 嚴格模式下為發布阻斷。耗時: {:?}",
                elapsed
            );
            std::process::exit(1);
        }
        println!(
            " [Rocq 9.2 驗證結論]: 理論編譯通過;rocqchk 微內核核檢【未執行】,不宣稱機械核檢通過。耗時: {:?}",
            elapsed
        );
        std::process::exit(0);
    } else if strict {
        eprintln!(
            " [Rocq 9.2 驗證結論]: rocq 缺席 — 僅完成理論合成與靜態校驗,零機械核檢。嚴格模式下為發布阻斷。耗時: {:?}",
            elapsed
        );
        std::process::exit(1);
    } else {
        println!(
            " [Rocq 9.2 驗證結論]: rocq 缺席 — 僅完成理論合成與靜態校驗,【零】機械核檢執行。耗時: {:?}",
            elapsed
        );
        std::process::exit(0);
    }
}
