//! l9newman —— 機械的 Newman 通道驅動程序(§4.3)。
//!
//! 對每個菜單機械驗證:
//!   (T8) 終止性(SN)→ (T9) 局部合流(WCR)→ 唯一正規形。
//! `CommutativeTrim` 應全綠;`Naive` 如實報告反例(側條件是定律的載體)。
//!
//! 運行:`cargo run --bin l9newman`。

use cl0r0::l9newman::newman_check;
use cl0r0::rep::{Menu, Policy};

fn main() {
    // 演示域:n 個事件、坐標 < max_coord、閉包/窮舉深度 depth
    const N_EVENTS: usize = 3;
    const MAX_COORD: u32 = 4;
    const DEPTH: usize = 8;

    let mut any_failure = false;
    for menu in [Menu::CommutativeTrim, Menu::Naive] {
        let report = newman_check(menu, Policy::Guarded, N_EVENTS, MAX_COORD, DEPTH);
        println!("======================================================================");
        println!(" 菜單:{}", report.menu.label());
        println!(
            " 狀態 {} | 臨界對 {} | L8 違反 {} | 不可回合 {} | 唯一正規形狀態 {} / {}",
            report.states,
            report.critical_pairs,
            report.l8_violations.len(),
            report.non_joinable.len(),
            report.unique_nf_states,
            report.states
        );
        println!(" 結論:{}", report.conclusion);
        println!("======================================================================");

        let expected_ok = matches!(menu, Menu::CommutativeTrim);
        let actually_ok = report.l8_violations.is_empty()
            && report.non_joinable.is_empty()
            && report.multi_nf.is_empty();
        if expected_ok != actually_ok {
            any_failure = true;
            eprintln!(
                "FAIL [{}]: 預期機械驗證 {},實際 {}",
                report.menu.label(),
                if expected_ok { "通過" } else { "反例" },
                if actually_ok { "通過" } else { "反例" }
            );
        }
    }

    if any_failure {
        std::process::exit(1);
    }
}
