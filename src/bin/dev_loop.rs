//! dev_loop —— 開發閉環看板機核裁判(DEV_LOOP.md 章程 §4 硬規則的強制者)。
//!
//! 讀取 BACKLOG.md(單一真相),檢查閉環不變式:
//! * WIP 上限:building + verifying ≤ 2(一次一件、做完關門);
//! * done 必有證據;planned/building/verifying 必有驗收門;
//! * 狀態/隊合法且 ID 唯一;proposed 堆積 ≤ 5(看板不得變倉庫)。
//!
//! 任一違規 ⇒ 非零退出(CI 擋下)。
//!
//! 運行: `cargo run --bin dev_loop`

use std::collections::BTreeSet;
use std::fs;

/// 合法狀態機(章程 §3)
const VALID_STATES: [&str; 6] = [
    "proposed",
    "planned",
    "building",
    "verifying",
    "done",
    "parked",
];
/// 三隊(章程 §1)
const VALID_TEAMS: [&str; 3] = ["語法語義", "證明證書", "品質基建"];
/// WIP 硬上限:building + verifying
const WIP_MAX: usize = 2;
/// proposed 堆積上限
const PROPOSED_MAX: usize = 5;

/// 一條看板項(欄位:ID|事項|隊|狀態|規模|驗收門|證據)
#[derive(Clone, Debug)]
struct Item {
    id: String,
    name: String,
    team: String,
    state: String,
    gate: String,
    evidence: String,
}

/// 解析看板:只認「以 | 開頭、ID 欄以 DL- 開頭」的表格列。
fn parse_board(md: &str) -> Result<Vec<Item>, String> {
    let mut items = Vec::new();
    for line in md.lines() {
        let t = line.trim();
        if !t.starts_with('|') || !t.contains("DL-") {
            continue;
        }
        let cols: Vec<&str> = t.split('|').map(|c| c.trim()).collect();
        // cols[0] 與尾欄為空;cols[1]=ID … cols[7]=證據
        if cols.len() < 8 || !cols[1].starts_with("DL-") {
            continue;
        }
        items.push(Item {
            id: cols[1].to_string(),
            name: cols[2].to_string(),
            team: cols[3].to_string(),
            state: cols[4].to_string(),
            gate: cols[6].to_string(),
            evidence: cols[7].to_string(),
        });
    }
    if items.is_empty() {
        return Err("看板無任何 DL- 條目(BACKLOG.md 遺失或格式壞?)".to_string());
    }
    Ok(items)
}

fn is_empty_cell(s: &str) -> bool {
    s.is_empty() || s == "—"
}

/// 檢查全部不變式,回傳違規清單(空 = 通過)。
fn check(items: &[Item]) -> Vec<String> {
    let mut v = Vec::new();
    let mut ids = BTreeSet::new();
    for it in items {
        if !ids.insert(it.id.clone()) {
            v.push(format!("{}: ID 重複", it.id));
        }
        if !VALID_STATES.contains(&it.state.as_str()) {
            v.push(format!(
                "{}: 非法狀態「{}」(合法:{})",
                it.id,
                it.state,
                VALID_STATES.join("/")
            ));
        }
        if !VALID_TEAMS.contains(&it.team.as_str()) {
            v.push(format!(
                "{}: 非法隊「{}」(合法:{})",
                it.id,
                it.team,
                VALID_TEAMS.join("/")
            ));
        }
        if matches!(it.state.as_str(), "planned" | "building" | "verifying")
            && is_empty_cell(&it.gate)
        {
            v.push(format!(
                "{}: 狀態 {} 但缺驗收門(章程:先定義做完的樣子)",
                it.id, it.state
            ));
        }
        if it.state == "done" && is_empty_cell(&it.evidence) {
            v.push(format!("{}: done 但缺證據(commit/CI run/門禁輸出)", it.id));
        }
    }
    let wip = items
        .iter()
        .filter(|i| i.state == "building" || i.state == "verifying")
        .count();
    if wip > WIP_MAX {
        v.push(format!(
            "WIP 超限:{}/{}(章程:一次一件、做完關門)",
            wip, WIP_MAX
        ));
    }
    let prop = items.iter().filter(|i| i.state == "proposed").count();
    if prop > PROPOSED_MAX {
        v.push(format!(
            "proposed 堆積:{}/{}(先 park/drop 再收新)",
            prop, PROPOSED_MAX
        ));
    }
    v
}

/// 看板路徑:cwd 優先,退回編譯期 crate 根。
fn board_path() -> std::path::PathBuf {
    let cwd = std::path::Path::new("BACKLOG.md");
    if cwd.exists() {
        return cwd.to_path_buf();
    }
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/BACKLOG.md")).to_path_buf()
}

fn main() {
    let path = board_path();
    let md = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[dev_loop] 無法讀取看板 {}: {}", path.display(), e);
            std::process::exit(1);
        }
    };
    let items = match parse_board(&md) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("[dev_loop] {}", e);
            std::process::exit(1);
        }
    };
    println!("====== 開發閉環看板檢查({})======", path.display());
    for it in &items {
        let brief: String = it.name.chars().take(30).collect();
        println!("  {:<7} {:<9} {:<8} {}", it.id, it.state, it.team, brief);
    }
    let violations = check(&items);
    let wip = items
        .iter()
        .filter(|i| i.state == "building" || i.state == "verifying")
        .count();
    let done = items.iter().filter(|i| i.state == "done").count();
    println!(
        "------ 小計:{} 項 · done {} · WIP {}/{} · proposed ≤{} ------",
        items.len(),
        done,
        wip,
        WIP_MAX,
        PROPOSED_MAX
    );
    if violations.is_empty() {
        println!("[dev_loop] 0 違規 — 閉環不變式成立。");
    } else {
        for v in &violations {
            eprintln!("[dev_loop] 違規:{}", v);
        }
        eprintln!(
            "[dev_loop] {} 項違規 — 章程見 DEV_LOOP.md §4。",
            violations.len()
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OK_BOARD: &str = "\
| ID | 事項 | 隊 | 狀態 | 規模 | 驗收門 | 證據 |
|----|------|----|------|------|--------|------|
| DL-001 | 示例甲 | 品質基建 | planned | M | 全綠 | — |
| DL-002 | 示例乙 | 證明證書 | done | S | 全綠 | commit abc123 |
";

    fn item(id: &str, team: &str, state: &str, gate: &str, ev: &str) -> Item {
        Item {
            id: id.to_string(),
            name: "測試項".to_string(),
            team: team.to_string(),
            state: state.to_string(),
            gate: gate.to_string(),
            evidence: ev.to_string(),
        }
    }

    #[test]
    fn parses_only_dl_rows() {
        let items = parse_board(OK_BOARD).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "DL-001");
        assert_eq!(items[0].state, "planned");
        assert_eq!(items[1].team, "證明證書");
    }

    #[test]
    fn empty_board_is_error() {
        assert!(parse_board("# 空看板\n").is_err());
    }

    #[test]
    fn valid_board_has_no_violations() {
        let items = parse_board(OK_BOARD).unwrap();
        assert!(check(&items).is_empty());
    }

    #[test]
    fn wip_over_limit_flagged() {
        let items: Vec<Item> = (0..3)
            .map(|i| {
                item(
                    &format!("DL-{:03}", i + 1),
                    "品質基建",
                    "building",
                    "全綠",
                    "—",
                )
            })
            .collect();
        assert!(check(&items).iter().any(|v| v.contains("WIP 超限")));
    }

    #[test]
    fn done_without_evidence_flagged() {
        let items = vec![item("DL-001", "品質基建", "done", "全綠", "—")];
        assert!(check(&items).iter().any(|v| v.contains("缺證據")));
    }

    #[test]
    fn active_without_gate_flagged() {
        let items = vec![item("DL-001", "品質基建", "building", "—", "—")];
        assert!(check(&items).iter().any(|v| v.contains("缺驗收門")));
    }

    #[test]
    fn bad_state_team_and_duplicate_id_flagged() {
        let items = vec![
            item("DL-001", "品質基建", "doing", "全綠", "—"),
            item("DL-001", "雜軍", "planned", "全綠", "—"),
        ];
        let v = check(&items);
        assert!(v.iter().any(|x| x.contains("非法狀態")));
        assert!(v.iter().any(|x| x.contains("非法隊")));
        assert!(v.iter().any(|x| x.contains("ID 重複")));
    }

    #[test]
    fn proposed_pileup_flagged() {
        let items: Vec<Item> = (0..6)
            .map(|i| {
                item(
                    &format!("DL-{:03}", i + 1),
                    "品質基建",
                    "proposed",
                    "—",
                    "—",
                )
            })
            .collect();
        assert!(check(&items).iter().any(|v| v.contains("proposed 堆積")));
    }
}
