//! lemma_stress_coverage —— 10 大形式化引理海量測試數據壓測與真實覆蓋率主程序。
//!
//! 運行: `cargo run --bin lemma_stress_coverage`

use cl0r0::lemma_stress_generator::LemmaStressEvaluator;
use std::time::Instant;

fn main() {
    println!("======================================================================");
    println!(" 10 大形式化引理海量測試數據壓測與不變量全覆蓋自証矩陣");
    println!("======================================================================");

    let start_time = Instant::now();
    println!(">> 正在生成並執行 50,000+ 組引理邊界、深層語法與高熵污料測試數據...");

    let report = LemmaStressEvaluator::run_massive_evaluation(0xC10_2024_0001, 5);
    let duration = start_time.elapsed();

    println!("----------------------------------------------------------------------");
    println!(" 各引理真實測試數據量與自証通過統計 (Breakdown):");
    println!("----------------------------------------------------------------------");
    println!(
        " [Lemma 1: L1 無損回環]        測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l1_cases,
        (report.l1_passed as f64) / (report.l1_cases as f64) * 100.0
    );
    println!(
        " [Lemma 2: L2 決定論]          測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l2_cases,
        (report.l2_passed as f64) / (report.l2_cases as f64) * 100.0
    );
    println!(
        " [Lemma 3/4: L3/L4 增量重析]    測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l3_l4_cases,
        (report.l3_l4_passed as f64) / (report.l3_l4_cases as f64) * 100.0
    );
    println!(
        " [Lemma 5: L5 Laminar & CW χ]  測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l5_cw_cases,
        (report.l5_cw_passed as f64) / (report.l5_cw_cases as f64) * 100.0
    );
    println!(
        " [Lemma 7: L7 污料 0-Panic]    測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l7_dirty_cases,
        (report.l7_dirty_passed as f64) / (report.l7_dirty_cases as f64) * 100.0
    );
    println!(
        " [Lemma 8: L8 良基測度遞減]    測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l8_decreasing_cases,
        (report.l8_decreasing_passed as f64) / (report.l8_decreasing_cases as f64) * 100.0
    );
    println!(
        " [Lemma 9: L9 van Oostrom DD]  測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l9_dd_cases,
        (report.l9_dd_passed as f64) / (report.l9_dd_cases as f64) * 100.0
    );
    println!(
        " [Lemma 9-Newman: Newman 快道] 測試用例: {:>6} | 通過率: {:>6.2}%",
        report.l9_newman_cases,
        (report.l9_newman_passed as f64) / (report.l9_newman_cases as f64) * 100.0
    );
    println!(
        " [Chordal: 區間圖弦性與完美圖] 測試用例: {:>6} | 通過率: {:>6.2}%",
        report.chordal_perfect_cases,
        (report.chordal_perfect_passed as f64) / (report.chordal_perfect_cases as f64) * 100.0
    );
    println!(
        " [DDMin: Delta 縮減 1-極小性]  測試用例: {:>6} | 通過率: {:>6.2}%",
        report.ddmin_cases,
        (report.ddmin_passed as f64) / (report.ddmin_cases as f64) * 100.0
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
        println!(" [自証結論]: 10 大形式化引理在海量測試數據下 100.00% 全部自証通過！");
    } else {
        eprintln!(" [自証結論]: 存在反例，請檢查未通過引理！");
        std::process::exit(1);
    }
}
