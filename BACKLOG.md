# BACKLOG — 開發閉環看板(單一真相)

> 裁判:`cargo run --bin dev_loop`(CI 強制)。章程與狀態機見 DEV_LOOP.md。
> 欄位:`ID | 事項 | 隊 | 狀態 | 規模 | 驗收門 | 證據`。狀態:proposed / planned / building / verifying / done / parked。

| ID | 事項 | 隊 | 狀態 | 規模 | 驗收門 | 證據 |
|----|------|----|------|------|--------|------|
| DL-000 | 開發閉環本體:章程 DEV_LOOP + 看板 BACKLOG + 裁判 bin dev_loop + CI 接線 + 文檔登記 | 品質基建 | done | S | dev_loop 0 違規;fmt/clippy/test 全綠;CI 全綠;README/CONTRIBUTING/ARCHITECTURE/ATLAS 四處接線 | 本提交;CI 全綠即補 run id |
| DL-001 | 覆蓋率冷點補測(圖鑑 D-1):ast.rs 41.2%、tactic_scheduler.rs 41.5%、l9newman.rs 53.6%、rustc_json.rs 58.0%、rep.rs 62.2% 五檔拉至各 ≥55%;允許拆兩片執行(片1=ast+tactic_scheduler、片2=其餘三檔) | 品質基建 | planned | M | llvm-cov 逐檔 ≥55% 且全庫不低於 71.3%;cargo test 全綠;clippy 0 | — |
| DL-002 | unwrap 訊息化第一批(圖鑑 D-3):ari_export.rs(11 處)、maude_engine.rs(10 處)裸 unwrap 改 expect 帶「不變式:…」訊息 | 品質基建 | planned | S | 兩檔 0 裸 unwrap(grep 見證);fmt/clippy/test 全綠 | — |
| DL-003 | 覆蓋率門檻上調 65 → 72(圖鑑 D-4;依賴 DL-001 完成後方可執行) | 品質基建 | planned | S | ci.yml --fail-under-lines 72;CI 全綠 | — |
| DL-004 | bin 主體邏輯下沉 lib 第一批(圖鑑 D-2):verify_all(277 行)與 ci_verify(260 行)決策層抽為 lib 可測函數 | 品質基建 | proposed | M | 下沉函數有單測;兩 bin 行數各 −30%;全量門禁綠 | — |
| DL-005 | CI 裝 Rocq 9.2 消滅 Gate 8 SKIPPED(圖鑑 D-6;若 CI 環境不可行,出 ADR 記錄替代路線) | 證明證書 | proposed | M | CI Gate 8 = Proven;或 ADR-009 替代方案獲產品負責人驗收 | — |

## 凍結規則(章程 §4)

DL-001…DL-005 清空前,不開新功能(阻塞修復與文檔除外);解凍權在產品負責人。

## 循環日誌

| 循環 | 日期 | 完成項 | CI |
|------|------|--------|-----|
| 0 | 2026-09-05 | DL-000(閉環本體) | 本輪 |
