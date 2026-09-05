# BACKLOG — 開發閉環看板(單一真相)

> 裁判:`cargo run --bin dev_loop`(CI 強制)。章程與狀態機見 DEV_LOOP.md。
> 欄位:`ID | 事項 | 隊 | 狀態 | 規模 | 驗收門 | 證據`。狀態:proposed / planned / building / verifying / done / parked。

| ID | 事項 | 隊 | 狀態 | 規模 | 驗收門 | 證據 |
|----|------|----|------|------|--------|------|
| DL-000 | 開發閉環本體:章程 DEV_LOOP + 看板 BACKLOG + 裁判 bin dev_loop + CI 接線 + 文檔登記 | 品質基建 | done | S | dev_loop 0 違規;fmt/clippy/test 全綠;CI 全綠;README/CONTRIBUTING/ARCHITECTURE/ATLAS 四處接線 | commit 4ca8aa6;CI run 33970482311 全綠(15 步) |
| DL-001 | 覆蓋率冷點補測(圖鑑 D-1):ast.rs 41.2%、tactic_scheduler.rs 41.5%、l9newman.rs 53.6%、rustc_json.rs 58.0%、rep.rs 62.2% 五檔拉至各 ≥55%;允許拆兩片執行(片1=ast+tactic_scheduler、片2=其餘三檔) | 品質基建 | done | M | llvm-cov 逐檔 ≥55% 且全庫不低於 71.3%;cargo test 全綠;clippy 0 | 本地 llvm-cov:ast 91.2%/ts 98.0%/l9 95.8%/rj 95.7%/rep 97.7%,lib 總 75.7%;165 測試綠;**並修得兩處潛伏 bug**(extract 作用域棧彈空 ⇒ 0 事件;借鏈分支不 walk ⇒ &mut 事件缺席) |
| DL-002 | unwrap 訊息化第一批(圖鑑 D-3):ari_export.rs(11 處)、maude_engine.rs(10 處)裸 unwrap 改 expect 帶「不變式:…」訊息 | 品質基建 | done | S | 兩檔 0 裸 unwrap(grep 見證);fmt/clippy/test 全綠 | grep -c unwrap() 兩檔皆 0(21 處 → expect 帶「不變式」訊息);165 測試綠 |
| DL-003 | 覆蓋率門檻上調 65 → 72(圖鑑 D-4;依賴 DL-001 完成後方可執行) | 品質基建 | done | S | ci.yml --fail-under-lines 72;CI 全綠 | ci.yml 門檻 65→72;CI run 33971501247 全綠,workspace 實測 75.4%(9518/12630)> 72 |
| DL-004 | bin 主體邏輯下沉 lib 第一批(圖鑑 D-2):verify_all(277 行)與 ci_verify(260 行)決策層抽為 lib 可測函數 | 品質基建 | done | M | 下沉函數有單測;兩 bin 行數各 −30%;全量門禁綠 | selfcheck.rs 671 行(決策層+Ledger+17 門禁函數+4 單測);verify_all 438→122(−72%);ci_verify 396→90(−77%);169 測試綠 |
| DL-005 | CI 裝 Rocq 9.2 消滅 Gate 8 SKIPPED(圖鑑 D-6;若 CI 環境不可行,出 ADR 記錄替代路線) | 證明證書 | done | M | CI Gate 8 = Proven;或 ADR-009 替代方案獲產品負責人驗收 | CI run 33996272152:rocq-core 9.2.0 裝成;verify_all 10/10 Proven·0 SKIP;ci_verify 7/7;rocqchk「Modules were successfully checked」 |
| DL-006 | unwrap 訊息化第二批(圖鑑 D-3 餘量):parse 6、ast 7、macro_lab 7、polonius_bridge 6、r0 2、rule_labeling 1、lex 1 共 30 處裸 unwrap → expect 帶逐點不變式訊息 | 品質基建 | done | S | 全庫非測試碼裸 unwrap = 0(grep 見證);fmt/clippy/test 全綠 | 全庫掃描 0 處(parse 6/ast 7/macro_lab 7/polonius 6/r0 2/rule_labeling 1/lex 1 共 30 處→expect 逐點不變式;r0 詞界判斷改 is_none_or);169 測試綠 |

## 凍結規則(章程 §4)

DL-001…DL-005 清空前,不開新功能(阻塞修復與文檔除外);解凍權在產品負責人。

## 循環日誌

| 循環 | 日期 | 完成項 | CI |
|------|------|--------|-----|
| 0 | 2026-09-05 | DL-000(閉環本體) | run 33970482311 ✓ |
| 1 | 2026-09-05 | DL-001、DL-002、DL-003(3 完成 ≥ 3 開工) | run 33971501247 ✓ |
| 2 | 2026-09-05 | DL-004、DL-005(2 完成 ≥ 2 開工;CI 首次 10/10+7/7 零 SKIP) | run 33996272152 ✓ |
| 3 | 2026-09-05 | DL-006(1 完成 ≥ 1 開工;D-3 全清) | run 33997238431 ✓ |
