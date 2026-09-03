//! lemma_stress_coverage —— 18 大形式化引理海量測試數據壓測與真實覆蓋率主程序。
//!
//! 運行: `cargo run --bin lemma_stress_coverage`

use cl0r0::lemma_stress_generator::LemmaStressEvaluator;
use std::time::Instant;

fn main() {
    println!("======================================================================");
    println!(" 18 大形式化引理海量測試數據壓測與不變量全覆蓋自証矩陣");
    println!("======================================================================");

    let start_time = Instant::now();
    println!(">> 正在生成並執行 80,000+ 組引理邊界、深層語法與高熵污料測試數據...");

    let report = LemmaStressEvaluator::run_massive_evaluation(0xC10_2024_0001, 5);
    let duration = start_time.elapsed();

    println!("----------------------------------------------------------------------");
    println!(" 各引理真實測試數據量與自証通過統計 (Breakdown):");
    println!("----------------------------------------------------------------------");
    println!(
        " [L1: 無損回環 (Lossless Roundtrip)]   測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l1_cases,
        (report.l1_passed as f64) / (report.l1_cases as f64) * 100.0
    );
    println!(
        " [L2: CST 決定論 (Determinism)]        測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l2_cases,
        (report.l2_passed as f64) / (report.l2_cases as f64) * 100.0
    );
    println!(
        " [L3/4: 增量重析與子樹重用]            測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l3_l4_cases,
        (report.l3_l4_passed as f64) / (report.l3_l4_cases as f64) * 100.0
    );
    println!(
        " [L5: Laminarity & CW-複形 Euler χ=1]  測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l5_cw_cases,
        (report.l5_cw_passed as f64) / (report.l5_cw_cases as f64) * 100.0
    );
    println!(
        " [L6: 具名投影同態引理]                測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l6_cases,
        (report.l6_passed as f64) / (report.l6_cases as f64) * 100.0
    );
    println!(
        " [L7: ERROR 全化 0-Panic 與極大良構]    測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l7_dirty_cases,
        (report.l7_dirty_passed as f64) / (report.l7_dirty_cases as f64) * 100.0
    );
    println!(
        " [L8: 良基測度字典序嚴格遞減]          測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l8_decreasing_cases,
        (report.l8_decreasing_passed as f64) / (report.l8_decreasing_cases as f64) * 100.0
    );
    println!(
        " [L9-DD: van Oostrom 局部山谷合流]     測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l9_dd_cases,
        (report.l9_dd_passed as f64) / (report.l9_dd_cases as f64) * 100.0
    );
    println!(
        " [L9-Newman: Newman 快速通道]          測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l9_newman_cases,
        (report.l9_newman_passed as f64) / (report.l9_newman_cases as f64) * 100.0
    );
    println!(
        " [L10-KB: Knuth-Bendix 臨界對可接合]   測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l10_kb_cases,
        (report.l10_kb_passed as f64) / (report.l10_kb_cases as f64) * 100.0
    );
    println!(
        " [L11: 1D 區間圖弦性與完美圖定理]      測試用例: {:>6} | 通過率: {:>6.2}%",
        report.chordal_perfect_cases,
        (report.chordal_perfect_passed as f64) / (report.chordal_perfect_cases as f64) * 100.0
    );
    println!(
        " [L12: 雙射跨度單子逆同態保真]        測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l12_monad_cases,
        (report.l12_monad_passed as f64) / (report.l12_monad_cases as f64) * 100.0
    );
    println!(
        " [L13: Delta Debugging 縮減 1-極小性]  測試用例: {:>6} | 通過率: {:>6.2}%",
        report.ddmin_cases,
        (report.ddmin_passed as f64) / (report.ddmin_cases as f64) * 100.0
    );
    println!(
        " [L14: Polonius Datalog 不動點等價]    測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l14_polonius_cases,
        (report.l14_polonius_passed as f64) / (report.l14_polonius_cases as f64) * 100.0
    );
    println!(
        " [L15: MIR 降階與 Def-Use 活躍決定論]  測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l15_mir_cases,
        (report.l15_mir_passed as f64) / (report.l15_mir_cases as f64) * 100.0
    );
    println!(
        " [L16: OOPSLA 2025 Reborrow 守恆]      測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l16_reborrow_cases,
        (report.l16_reborrow_passed as f64) / (report.l16_reborrow_cases as f64) * 100.0
    );
    println!(
        " [L17: Aeneas 反向函數語義等價]        測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l17_aeneas_cases,
        (report.l17_aeneas_passed as f64) / (report.l17_aeneas_cases as f64) * 100.0
    );
    println!(
        " [L18: 持久化結構共享差分重析 (>=92%)] 測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l18_diff_share_cases,
        (report.l18_diff_share_passed as f64) / (report.l18_diff_share_cases as f64) * 100.0
    );
    println!("----------------------------------------------------------------------");
    println!(" 海量測試匯總:");
    println!("   - 總計測試數據量:       {} 個樣本", report.total_tested);
    println!("   - 成功自証通過數:       {} 個樣本", report.total_passed);
    println!("   - 總體自証成功率:       {:.4}%", report.success_rate);
    println!("   - 壓測執行總耗時:       {:?}", duration);
    println!(
        "   - 單樣本平均驗證延遲:   {:.2} µs/case",
        (duration.as_micros() as f64) / (report.total_tested as f64)
    );
    println!("======================================================================");

    if report.total_passed == report.total_tested {
        println!(" [自証結論]: 18 大形式化引理在海量測試數據下 100.00% 全部自証通過！");
    } else {
        eprintln!(" [自証結論]: 存在反例，請檢查未通過引理！");
        std::process::exit(1);
    }
}
