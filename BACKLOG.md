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

| DL-007 | 覆蓋率提升:<74% 七檔拉至各 ≥74%(isabelle_export 66.3 / reparse_verifier 66.7 / rocq_export 67.5 / lsp_bridge 70.1 / lemmas 71.5 / variance_dropck_ub 71.2 / proof_resources 71.7) | 品質基建 | done | M | llvm-cov 逐檔 ≥74%;全量門禁綠 | 本地 llvm-cov:isabelle 100/reparse 88.2/rocq 75.7/lsp 99.3/lemmas 82.0/variance 98.4/proof_resources 99.7;全庫 84.0%;lib +23 測試;fmt/clippy(0 warn)/test 200/200/dev_loop 0 違規;CI run 33999567428 全綠(lcov 複核:isabelle 100/reparse 88.0/rocq 76.6/lsp 99.3/lemmas 81.9/variance 98.4/proof_resources 100;全庫 84.62%) |
| DL-008 | bin 下沉第 2 批(圖鑑 D-2 續):cl0r0 九律自証引擎 + fuzz 屬性套件引擎下沉 lib 並補單測 | 品質基建 | done | M | 下沉函數有單測;兩 bin 行數各 −40%;全量門禁綠 | 新 lib fuzz_engine.rs(704 行,8 單測);bin/fuzz.rs 423→44(−89.6%)、bin/cl0r0.rs 265→155(−41.5%),兩 bin 實跑退出碼 0;下沉時修復潛伏投影 bug(初始紅邊 7→5 步→0);CI run 33999567428 全綠 |
| DL-009 | 專有化法律地基(產品化第一步):LICENSE 改專有 EULA(撤銷 Cargo.toml MIT 標示)、THIRD_PARTY_NOTICES、全部 .rs 檔頭版權聲明 | 產品化 | done | S | Cargo.toml 不再標 MIT;LICENSE+NOTICES 入 repo;67 個 .rs 檔頭全有聲明;門禁全綠 | Cargo.toml license="MIT"→license-file="LICENSE";專有 LICENSE(DRAFT 待律師審+版權人名回填)入 repo;THIRD_PARTY_NOTICES(運行時零依賴聲明+工具鏈清單)入 repo;70 個 .rs 檔頭加版權聲明;dev_loop 合法隊增「產品化」(章程同步四隊);fmt/clippy 0/test 200 全綠;[COPYRIGHT HOLDER] placeholder 待產品負責人回填 |
| DL-010 | workspace 5-crate 重構(計畫先行:WORKSPACE_PLAN.md 定案映射與依賴方向,獲產品負責人確認後方動碼) | 產品化 | done | L | 依賴方向編譯器強制;200 測試全綠;CI 綠;Atlas/文檔同步 | 計畫樹 WORKSPACE_PLAN.md 定案:51 模組映射 C1 syntax(9)/C2 mir(3)/C3 ars(23)/C4 cert(5)/C5 facade(11+bins);依賴矩陣實測無環;5 粘連點對策(K1 巨集/K2 cpf_cert/K3 patch_engine/K4 testkit/K5 路徑替換);產品負責人已批准(2026-09-06);C1..C5 全數遷移完畢:crates/cl0r0-{syntax,mir,ars,cert}+root facade;workspace 200 測試綠;clippy --workspace 0 warning;5 bin 實跑 ✓;ci.yml clippy/test 補 --workspace;CI run 34008381873 全綠(workspace lcov 覆蓋率 84.62%≥72);5 commit 分步遷移(33af21c/5725184/6f6469b 系列) |
| DL-011 | .wasi 跨平台打包 PoC —— 產品負責人指示:押後至 beta 版本鎖定時再決定 | 產品化 | parked | M | (凍結中;解凍權在產品負責人) | 2026-09-06 產品負責人凍結 |
| DL-012 | 內部規格文書包(data room):架構/介面規格/證書格式對接/版本紀錄/質量証據匯編;依賴 DL-010 定案後執行以免重寫 | 產品化 | done | M | 文書包可獨立成冊交第三方審閱 | commit f774a4f;CI run 34025271472 全綠;docs/ 六檔:DATA_ROOM_INDEX/SPEC_ARCHITECTURE/SPEC_INTERFACE/SPEC_CERTIFICATES/VERSION_RECORD/QUALITY_EVIDENCE;200 測試/clippy 0/fmt ✓/dev_loop 0 違規 |
| DL-013 | Isabelle 導出升格(F-04 第二階段):定理由註釋草案升為顯式 sorry 宣言(合法 Isabelle/HOL 語法+數據實錄定義)+結構良構機檢(structural_audit);完整 Isabelle 証明仍屬後續,如實申報 | 證明證書 | done | M | 定理陳述脫離註釋;sorry 計數=定理計數機檢;全量門禁綠 | 定理脫離註釋→顯式 sorry 宣言(僅標準庫,零 IsaFoR);structural_audit 嵌套註釋詞法機檢(捉出真實詞法坑:代碼區孤立 *));+1 測試;isabelle_export 覆蓋 100%;218 測試/clippy 0/fmt ✓;commit 29bdfcd;CI run 34027780280 全綠 |
| DL-014 | bin 整合測試(零第三方):tests/bins_smoke.rs 以 CARGO_BIN_EXE_* 驅動 17 個 bin,斷言退出碼+特徵輸出,補量測空洞 | 品質基建 | done | M | 17 bin 各 ≥1 整合測試;llvm-cov bin 檔不再全 0%;全量門禁綠 | tests/bins_smoke.rs 17 測試(零第三方,CARGO_BIN_EXE_*;30s 超時保護;lsp 餵 LSP base protocol 真幀);bin 覆蓋 0%→47–100%(lsp_server 25.8→83.9);全庫 84.62→90.87%(CI lcov);218 測試;commit 29bdfcd;CI run 34027780280 全綠 |
| DL-015 | Miri/Stacked Borrows 差分對齊:UB 預言機與 SB 規則情景對照表(機檢差分測試+分歧登記);本環境 miri 元件缺席(ADR 如實記錄) | 證明證書 | done | M | 情景表機檢全綠;分歧登記完整;門禁全綠 | SB_SCENARIOS 5 情景機檢一致;SB_DIVERGENCES 4 項分歧登記(SB-D1 行為機檢);+3 測試;ADR-015 記錄 miri 缺席+100% safe Rust;commit 6bf52f8;CI 34030229681 全綠(全庫 90.76%) |
| DL-016 | 第三方 CPF 消費入口(D-2 續):cpf_import 解析+獨立複核(documented subset:DD/KB/正交),bin cpf_check;CeTA 完整互操作屬後續 | 證明證書 | done | M | 第三方樣本 Verified;篡改樣本 Rejected;不支援類型如實 Unsupported;門禁全綠 | crates/cl0r0-ars 新增 cpf_import(documented subset 四態判定;5 單測:第三方 DD Verified/環偏序 Rejected/未知類型 Unsupported/截斷+轉義+缺見證 Malformed/KB+正交 Verified);bin cpf_check(無參自演示 exit 0 實跑 ✓;退出碼 0/3/4/5/6);bins_smoke +1;227 測試/clippy 0/fmt ✓;commit 6bf52f8;CI 34030229681 全綠;插曲:首推 YAML 縮進失誤 34029365372 失敗(0 jobs),即修 8ea6ec1 轉綠——過程如實記錄 |
| DL-017 | multi-edit 增量重析等價補全(L3/L4):批次多編輯(2–5 個,含相鄰/邊界)與逐編輯、全量解析三方等價屬性測試 | 品質基建 | verifying | M | 批次=逐個=全量 sexp 等價;隨機種子 30 輪;重用率>0;門禁全綠 | multi_edit_tests 6 測試:2/5 編輯批次等價+重用、逐步≡批次(座標平移)、邊界+相鄰、30 輪隨機 K=2..4 屬性、未觸及 item 確定性重用;等價全數成立(引擎批次語義本已正確,測試補証);233 測試/clippy 0;待 CI |

## 凍結規則(章程 §4)

DL-001…DL-005 清空前,不開新功能(阻塞修復與文檔除外);解凍權在產品負責人。

## 循環日誌

| 循環 | 日期 | 完成項 | CI |
|------|------|--------|-----|
| 0 | 2026-09-05 | DL-000(閉環本體) | run 33970482311 ✓ |
| 1 | 2026-09-05 | DL-001、DL-002、DL-003(3 完成 ≥ 3 開工) | run 33971501247 ✓ |
| 2 | 2026-09-05 | DL-004、DL-005(2 完成 ≥ 2 開工;CI 首次 10/10+7/7 零 SKIP) | run 33996272152 ✓ |
| 3 | 2026-09-05 | DL-006(1 完成 ≥ 1 開工;D-3 全清) | run 33997238431 ✓ |
