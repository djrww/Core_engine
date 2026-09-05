//! dd_verify —— 递减图 (Decreasing Diagrams) 与 Newman 快速通道自证驱动程序。
//!
//! 运行: `cargo run --bin dd_verify`

use cl0r0::cpf_cert::{CPFCertificate, CertResult};
use cl0r0::dd_checker::{check_confluence_with_mode, CheckerMode, SNWitness};

fn main() {
    println!("======================================================================");
    println!(" CL0 / R₀ 遞減圖 (Decreasing Diagrams) 與 Newman 快速通道自証");
    println!("======================================================================");

    // D-01:狀態宇宙引用 testkit 夾具單一真相(60 配置,舊 54 狀態宇宙的超集)
    let states = cl0r0::testkit::fixtures::sample_states();

    // [1/3] 遞減圖模式
    println!(
        "[1/3] 正在核驗 {} 個狀態上的局部峰值遞減山谷會合性 (DD 軌道)...",
        states.len()
    );
    let report_dd = check_confluence_with_mode(&states, CheckerMode::DecreasingDiagrams, 6);
    println!("  總枚舉峰值數 (Local Peaks): {}", report_dd.total_peaks);
    println!(
        "  成功會合遞減山谷數 (Decreasing Valleys): {}",
        report_dd.decreasing_valleys_found
    );
    println!("  不可會合峰值數: {}", report_dd.non_joinable_peaks.len());
    if !report_dd.certified {
        eprintln!("FAIL: 發現不可會合峰值！");
        std::process::exit(1);
    }
    println!("  >> 遞減圖合流性機械驗證通過！ (100% Confluent)");

    // [2/3] Newman 快速通道
    println!("\n[2/3] 正在核驗 Newman 快速通道 (帶 SN 作用域見證)...");
    let witness = SNWitness::LivenessScopeBounded {
        max_span_len: 4,
        storages: 1,
    };
    let start_n = std::time::Instant::now();
    let report_newman = check_confluence_with_mode(
        &states,
        CheckerMode::Newman {
            sn_witness: witness.clone(),
        },
        6,
    );
    let dur_n = start_n.elapsed();
    println!(
        "  Newman 快速通道耗時: {:?} (出具 CPF-KB 短證: {})",
        dur_n,
        report_newman.cpf_kb_proof.is_some()
    );

    // [3/3] CPF 雙重證書核驗
    println!("\n[3/3] 正在核驗 CPF 證書 (DD 偏序 + KB 短證)...");
    // D-02:標號宇宙由 rule_labeling 單一真相導出
    let cert_dd = CPFCertificate::new_decreasing_diagrams(
        "CL0-CommutativeTrim-DD",
        cl0r0::rule_labeling::dd_label_universe(),
        cl0r0::rule_labeling::dd_strict_order_pairs(),
    );
    // F-03:KB 短證攜帶 Newman 通道機械實錄的臨界對會合見證
    let cert_kb = CPFCertificate::new_knuth_bendix(
        "CL0-KB-Short",
        &witness.description(),
        report_newman.kb_critical_pair_witnesses.clone(),
    );

    match (cert_dd.verify(), cert_kb.verify()) {
        (CertResult::Certified, CertResult::Certified) => {
            println!("  >> CPF 證書雙重核驗成功: CERTIFIED (DD 偏序無環 ∧ KB 短證合法)");
        }
        (err1, err2) => {
            eprintln!("FAIL: CPF 證書被拒絕: {:?}, {:?}", err1, err2);
            std::process::exit(1);
        }
    }

    println!("\n======================================================================");
    println!(" 機械自証全綠：L8 (良基偏序/SN 見證) ∧ L9 (合流唯一正規形) 雙通道通過");
    println!("======================================================================");
}
