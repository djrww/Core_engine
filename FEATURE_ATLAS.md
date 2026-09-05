# FEATURE_ATLAS — 全功能圖鑑與證書冊

> **版本**: v0.2.0 · 2026-09-05
> **範圍**: 48 個 lib 模組 + 17 個 bin = **65 項功能**(另有 10 個真巨集內嵌於 `macro_lab`)。
> **覆蓋率口徑**: CI run `33971501247`(commit `7772995`,DL-001 補測後)之 `coverage-lcov` 產物,lcov 逐檔 DA 行加總;**全庫行覆蓋率 75.4%**(門檻 72,DL-003 上調)。bin 檔顯示 0% 為量測空洞(`cargo llvm-cov` 只統計儀器化測試執行,`cargo run` 自証執行不計入),非真的零執行——CI 每輪都實跑全部 16 個 bin。
> **測試**: `cargo test --all-targets` = **165/165 通過**(lib 122 + bins/integration 43;DL-001 +34)。
> **認證**: 表2 登記 15 項有證書功能;CI 端到端 10 門禁(verify_all)+ 7 門禁(ci_verify)+ 14 門禁(macro_lab)。

---

## 表 1 · 全功能全景(64 項)

欄位:**功能**(原始檔)/ **可以做咩**(用途)/ **內嵌左啲咩**(核心算法·資料結構·定理)/ **語法層**(lex → parse → AST/TT → MIR → 語義/證明 所處階層)/ **語義係咩**(數學語義·依據定理)/ **覆蓋率**。

### A. lib 模組(48)

| # | 功能 | 可以做咩 | 內嵌左啲咩 | 語法層 | 語義係咩 | 覆蓋率 |
|---|------|----------|------------|--------|----------|--------|
| 1 | `lex.rs` | CL0 詞法分析,任意破損位元流 0-Panic | 平鋪 DFA、Bad-token 全化、utf8 步進 | **詞法層** | CL0 詞法完全正則(§5.1);非法輸入化為 Bad token 不炸 | 93.7% |
| 2 | `parse.rs` | 表面語法樹、增量重析、ERROR 全化恢復 | 棧式解析、子樹快照重用、SyncMode 錯誤恢復 | **語法層** | L1 無損回環;L3/L4 增量重析等價;L2 決定論 | 94.0% |
| 3 | `span.rs` | 位址映象 σ: V→ℤ×ℤ 半開區間 | Span 代數、位移函數 | **語法層**(幾何基石) | 連續性公理;L5 嵌套定理 | 88.5% |
| 4 | `edit.rs` | 編輯操作合成與位移 | 編輯 Monoid、結合律見證 | **語法層** | Edit monoid 恆等律/結合律 | 78.9% |
| 5 | `span_monad.rs` | 跨度單子(事實層↔AST 逆映射) | Monadic bind、逆同態 | **語法層** | 双向逆同態映射 | 94.4% |
| 6 | `ast.rs` | 語義區間、liveness 投影、衝突圖 | 區間圖 ⊂ 弦圖 ⊂ 完美圖結構(§3.2) | **語法→語義** | 區間圖著色語義;liveness 半開區間投影 | 91.2% |
| 7 | `diff_tree.rs` | 持久化結構共享語法樹、差分快照 | `Arc<DiffAstNode>` 不可變共享、脊椎重構 | **語法層** | L18:共享率 ≥92%(實測 95.08%)、突變語義無損 | 78.9% |
| 8 | `tree.rs` | 樹操作輔助:無損回環、具名投影檢驗 | `Tree` 方法輔助函數 | **語法層** | CW 複形歐拉公理(χ=1)檢驗 | 95.2% |
| 9 | `token_tree.rs` | 巨集用 Token 樹解析/渲染/匹配 | TT::Atom/Group、Frag 判別子、match_seq(貪婪+回退) | **巨集語法層**(借.md §0) | 模式匹配語義;Rep 零輪∈候選;telemetry 比較計數 | 84.2% |
| 10 | `macro_lab.rs` | 巨集七原則實驗室:展開鏈模擬+真巨集對照 | expand_chain(委派語義)、μ 良基遞減(munch_mu)、九系統 registry、P1–P7 門禁、complexity_report、**10 個真巨集**(cl0_safe_vec!/cl0_with_val!/cl0_laminar_scope!/cl0_count_tts!/cl0_double!/cl0_borrow_kind!/cl0_produce!/cl0_consume!/cl0_double_tt!反例/ScopeGuard) | **巨集層**(模型=語法樹;真巨集=語法擴展) | 互斥⇒合流、μ 遞減⇒終止、CPS、hygiene、let sharing、Tree 代換;Θ(n²) 實測 3.45 | 89.6% |
| 11 | `borrow_model.rs` | 借用組合模型:三算法+Datalog 判定 | κ-矩陣(4 格 3 衝突)、naive O(n²)/sweep O(n log n)/laminar 棧、MiniDatalog(§2 三規則逐字;env 快照回溯)、B1–B6 門禁 | **借用語義層** | 3·o²·n² 搜尋空間解剖;error=∅ ⟺ 無衝突;層狀族 depth×n | 98.2% |
| 12 | `rep.rs` | 修法菜單重寫系統:終止與合流檢查 | TRS、正規形、critical pairs | **語義層** | SN/CR(WCR+SN⇒CR) | 97.7% |
| 13 | `rep_dd.rs` | 遞減圖 ARS 抽象(修法菜單) | van Oostrom 規則標號、red_edges 衝突對 | **語義層** | Decreasing Diagrams 定理(局部⇒全局合流,免全局 SN) | 77.2% |
| 14 | `dd_checker.rs` | 遞減圖+Newman 快速通道核驗 | 偏序標號 Trim≺Split≺Runtime、SNWitness、峰值收斂檢查 | **語義/證書層** | DD 合流;SN∧WCR⇒CR;CPF-KB 短證出具 | 88.7% |
| 15 | `rule_labeling.rs` | 規則標號啟發式求解 | 良基偏序搜索、嚴格序對 | **語義層** | 標號存在 ⇒ DD 條件成立 | 90.5% |
| 16 | `l9newman.rs` | 機械 Newman 通道 | 作用域見證、局部 WCR 檢查 | **語義層** | Newman 引理(1942) | 95.8% |
| 17 | `r0.rs` | R₀ 實用載體(Rust 子集) | 附錄 B EBNF、LALR(1) 乾淨子集、`unsupported` 邊界防火牆 | **語法+語義層**(第二載體) | 實用載體語義;邊界明確拒絕而非誤判 | 81.5% |
| 18 | `r0_lower.rs` | R₀ → 事實層降階 | 語義保持降階、liveness 圖生成 | **語義層** | 降階同態保持語義 | 93.1% |
| 19 | `mir.rs` | MIR 中介表示:CFG/Move/Drop/NLL | 控制流圖、Place 投影、Move 數據流、NLL 借用檢查 | **語義層**(中介表示) | NLL 非詞法生命週期;Def-Use 活躍期 | 75.9% |
| 20 | `modular_contracts.rs` | 模組化函數契約、Reborrow 懸掛語義 | OOPSLA 2025 標準、懸掛/重活化棧、循環不動點求解器 | **借用語義層** | 契約組合語義;不動點存在唯一 | 97.7% |
| 21 | `variance_dropck_ub.rs` | Variance/Dropck/UB 核驗預言機 | 協變/逆變/不變推導、`#[may_dangle]` 針眼法則、UB 預言 | **類型語義層** | Rustonomicon 標準;Soundness 見證 | 71.2% |
| 22 | `polonius_bridge.rs` | Polonius Datalog 事實層雙向橋接 | EDB/IDB 事實生成、原生定點求解、`-Zpolonius` 回環 | **借用語義層** | Polonius 三規則;error 關係 = 借用錯誤 | 92.0% |
| 23 | `lemmas.rs` | 18 大形式化引理註冊表 | L1–L18 強類型見證(LemmaWitnessData)、機械證明 | **證書層** | 18 引理 100% 機械自証 | 71.5% |
| 24 | `cpf_cert.rs` | CPF 風格證書載體與核驗器 | CPF-DD(偏序無環)/CPF-KB(短證)雙重自檢 | **證書層** | 證書 = 可獨立複核的證據;CertResult 三態 | 97.8% |
| 25 | `ari_export.rs` | CoCo 2025/2026 ARI 格式導出 | ARI (Automated Rewriting Interface) 規範編碼 | **證書/交換層** | 交換格式語義(標準符合) | 100% |
| 26 | `isabelle_export.rs` | Isabelle/HOL 理論草稿導出 | CeTA/CPF XML 草稿生成 | **證書層**(外部) | Isabelle 理論語義;CeTA 可核驗性 | 66.3% |
| 27 | `rocq_export.rs` | Rocq 9.2 理論導出與微內核核檢 | `.v` → `rocq compile` → `.vo` → `rocqchk`;KernelCheck 三態 | **外部證明層** | 微內核獨立核驗(不可信導出、可信核检) | 67.5% |
| 28 | `creusot_export.rs` | Creusot/Why3+Z3 演繹驗證 | Pearlite 預言變量 (current,prophecy)、SMT goals 消解 | **外部證明層** | 預言變量演算;VC 100% 消解 = Proven | 96.5% |
| 29 | `proof_resources.rs` | Rust 類型作為證明資源 | Aeneas 反向函數、Creusot 預言模型、Prusti 分離邏輯 | **外部證明層** | 分離邏輯契約 | 71.7% |
| 30 | `tactics.rs` | Coq 風格自動戰術庫 | `congruence`/`lia`/`omega` 模擬 | **證明層** | 決策程序語義 | 99.3% |
| 31 | `tactic_scheduler.rs` | ARI-COPS 策略調度與對拍 | 策略隊列、命中率統計、NewmanFastPath | **證明工程層** | 策略調度啟發式 | 99.0% |
| 32 | `unification.rs` | 近線性一階合一 | DAG 項 union-find、occurs-check | **演算法層** |合一演算法完備性/健全性 | 81.3% |
| 33 | `dag_term.rs` | DAG 項表示與 Hash-Consing 池 | 指針共享、hash-cons、結構相等 O(1) | **演算法層** | DAG≡樹語義(共享透明) | 91.1% |
| 34 | `discrimination_tree.rs` | 辨別樹規則索引+指針倒排 | 倒排索引、首符號分派 | **演算法層** | 索引語義:不漏報 | 96.0% |
| 35 | `maude_engine.rs` | Maude 重寫邏輯引擎 | 等式重寫、LTL 模型檢查介面 | **語義/工程層** | 重寫邏輯語義 | 84.1% |
| 36 | `gen.rs` | 屬性測試輸入宇宙 | 定律×載體矩陣抽樣、Rng 種子確定性 | **測試層** | 窮舉/抽樣語義 | 88.2% |
| 37 | `shrink.rs` | 反例最小化(DDMin) | 不變量驅動 AST 剪枝 | **測試層** | Delta debugging:最小可重現 | 100% |
| 38 | `differential_checker.rs` | 跨引擎差分測試+黃金標準對賬 | 差分矩陣、DDMin 收斂、評分 | **測試層** | 差分審核評分 99.02% | 88.6% |
| 39 | `lemma_stress_generator.rs` | 79,000 樣本海量壓測生成+不變量評估 | 18 引理×輸入宇宙、不變量求值器 | **測試層** | 不變量 100% 保持 = 0 defect | 96.2% |
| 40 | `json_report.rs` | 結構化 JSON 診斷+自動修復管線 | 錯誤分類、修復建議、閉環驗證 | **工程層**(消費語法/語義產物) | 診斷語義;修復 = 證據驅動 | 86.4% |
| 41 | `lsp_bridge.rs` | LSP CodeAction 互動修法 | JSON-RPC 2.0 處理器 | **工程層** | LSP 協議語義 | 70.1% |
| 42 | `rustc_json.rs` | rustc JSON 診斷 → 重寫自動機 | 診斷解析、ARS 套用 | **工程層** | 外部診斷→內部 ARS 同化 | 95.7% |
| 43 | `tool_runner.rs` | 外部證明工具子進程封裝 | 單一真相封裝(審計 F-09/F-12/D-03) | **工程層** | 子進程語義:逾時/缺席三態 | 81.8% |
| 44 | `patch_engine.rs` | 端到端補丁合成閉環自驗 | 補丁生成+自驗 | **工程層** | 合成⇒驗證閉環 | 95.0% |
| 45 | `pipeline_synthesis.rs` | 五大組合深度合成引擎 | 五階段流水線編排 | **工程層** | 端到端組合語義 | 97.0% |
| 46 | `cert_generator_factory.rs` | 污料宇宙+證書批量生產 | 污料生成、工廠產線、髒輸入穩健性核驗 | **測試/證書層** | 認證流水線語義 | 95.8% |
| 47 | `testkit.rs` | 共享測試見證夾具 | fixtures 單一真相(審計 D-01) | **測試基建** | 見證共用語義 | 100% |
| 48 | `reparse_verifier.rs` | 增量重析等價驗證 | 快照重用等價檢查 | **測試層** | L3/L4 等價 | 66.7% |

### B. bin 執行程序(17)

| # | 功能(bin) | 可以做咩 | 內嵌左啲咩 | 語法層 | 語義係咩 | 覆蓋率 |
|---|------|----------|------------|--------|----------|--------|
| 49 | `cl0r0` | 主演示:九律檢查+幾何/重寫演示 | 九律檢查、幾何演示、R₀ 接線 | 全層演示 | 定律載體巡禮 | 0%* |
| 50 | `verify_all` | 端到端全量自証(10 門禁) | GATES 常量驅動、三態(Proven/Skipped/Failed)、`--strict` 發布模式 | 全層 | 門禁語義:SKIPPED≠PASSED(F-01) | 0%* |
| 51 | `ci_verify` | CI 專項 7 大門禁執行器 | 18 引理+壓測+合成+JSON+DAG+MIR+差分 七門禁串聯 | 全層 | CI 驗收閉環 | 0%* |
| 52 | `macro_lab` | 巨集七原則+借用組合證據鏈(14 門禁) | P1–P7+B1–B6+Θ(n²) 實測;`--verbose` 逐規則傾印 | 巨集/借用層 | 證據鏈語義:14/14 PASS | 0%* |
| 53 | `dd_verify` | 遞減圖+Newman 自証驅動 | 峰值會合演示、CPF-KB 短證出具 | 語義層 | DD+Newman 雙通道 | 0%* |
| 54 | `l9newman`(bin) | Newman 快速通道獨立驅動 | SNWitness 出具、耗時計量 | 語義層 | SN∧WCR⇒CR | 0%* |
| 55 | `rocq_verify` | Rocq 導出+微內核核檢主程序 | tool_runner 三態、rocqchk 呼叫 | 外部證明層 | 微內核複核 | 0%* |
| 56 | `creusot_verify` | Creusot/Why3 SMT 消解主程序 | Pearlite 契約、Z3 消解 | 外部證明層 | VC 消解語義 | 0%* |
| 57 | `cert_factory` | 污料宇宙+證書批量生產運行機 | 污料生成、認證產線 | 測試/證書層 | 批量認證 | 0%* |
| 58 | `coco_benchmark` | 國際合流基準壓測(CoCo) | 基準套件執行、計分 | 語義/交換層 | 基準語義 | 0%* |
| 59 | `fuzz` | 確定性種子屬性測試主程序 | 種子重現、屬性斷言 | 測試層 | 屬性測試語義 | 0%* |
| 60 | `fuzz_daemon` | 4 小時輪自動 Fuzzing 守護進程 | 定時排程、日誌 | 測試層 | 守護進程語義 | 0%* |
| 61 | `lemma_stress_coverage` | 79k 壓測+真實覆蓋率主程序 | 海量樣本、覆蓋率統計 | 測試層 | 壓測證書 | 0%* |
| 62 | `pipeline_runner` | 五大組合合成驗證程序 | 流水線編排執行 | 工程層 | 組合驗證 | 0%* |
| 63 | `dev_prover` | 開發階段引理取証+証物提取 | 証物打包、取証流水線 | 證明工程層 | 証物規範 | 0%* |
| 64 | `lsp_server` | 獨立 LSP 服務二進制 | JSON-RPC 伺服循環 | 工程層 | LSP 服務語義 | 0%* |
| 65 | `dev_loop` | 開發閉環看板機核裁判 | BACKLOG.md 解析 + 不變式檢查(WIP≤2/done 必有證據/proposed≤5) | 開發流程層(治理) | 閉環不變式語義;違規即非零退出 | 0%* |

\* bin 0% = 量測空洞(llvm-cov 不計 CI 的 `cargo run` 執行);CI 每輪實跑全 16 bin 且全綠。

---

## 表 2 · 有證書的功能 · 登記冊(15 項)

> 冊規:每條登記 = 證書形態 + 簽發/核驗通道 + 證據 + 現狀。「機內」= 庫內機械自証;「外部」= 第三方證明器獨立核驗;SKIPPED = 工具缺席如實申報(絕不偽稱 PASSED,審計 F-01)。

| # | 功能 | 證書形態 | 簽發/核驗通道 | 證據 | 現狀 |
|---|------|----------|----------------|------|------|
| 1 | `cpf_cert` | **CPF-DD**(遞減圖偏序無環)+ **CPF-KB**(KB 短證)雙證書 | 機內雙重自檢 | DD 無環 ∧ 26 對峰值 KB 短證見證全數合法 | ✅ CI Gate 5 CERTIFIED |
| 2 | `lemmas`(L1–L18) | 18 引理機械證明見證 | 機內(LemmaWitnessData 強類型) | 18/18 獲機器證明見證;79k 樣本不變量 100% | ✅ CI Gate 7 |
| 3 | `dd_checker` | 遞減圖會合證書 + CPF-KB 短證 | 機內 | 60 狀態/26 峰值收斂唯一正規形;短證 318µs | ✅ CI Gate 3+4 |
| 4 | `diff_tree`(L18) | 結構共享率見證證書 | 機內實測 | 共享率 95.08%(標桿 ≥92%)+ 逐位元組等價 | ✅ CI Gate 1+2 |
| 5 | `reparse_verifier` | 增量重析等價證書 | 機內 | 1000 樣本 100% 逐字節吻合 | ✅ CI Gate 1 |
| 6 | `rocq_export` | `.vo` + **rocqchk 微內核**核檢證書 | 外部(Rocq 9.2) | rocqchk 獨立複核 KernelCheck | ⏸ SKIPPED(本機/CI 無 rocq,如實申報;裝機即 Proven) |
| 7 | `creusot_export` | Why3+Z3 SMT 消解證書 | 外部(Why3+Z3) | SMT goals 100% Valid(預言變量契約) | ✅ CI Gate 9 Proven(CI 已裝 why3+z3) |
| 8 | `isabelle_export` | Isabelle/HOL 理論草稿(CeTA/CPF XML 可核驗形態) | 外部(Isabelle/CeTA 消費) | 導出態:理論草稿+CPF XML | 📤 導出就緒(待 CeTA 環境) |
| 9 | `ari_export` | CoCo 2025/2026 **ARI/COPS 交換格式**證書 | 交換標準符合 | ARI 規範編碼 100% 標準形 | ✅ CI(coco_benchmark) |
| 10 | `cert_generator_factory` | 污料宇宙穩健性+批量認證證書 | 機內 | 髒輸入 0 Panic;認證產線閉環 | ✅ CI(cert_factory) |
| 11 | `lemma_stress_generator` | 79,000 樣本壓測證書 | 機內 | 79k 樣本 0 defect、不變量 100% | ✅ CI(lemma_stress_coverage) |
| 12 | `differential_checker` | 差分審核評分證書 | 機內+黃金標準對賬 | 差分審核矩陣 99.02% 評分 | ✅ CI |
| 13 | `macro_lab`(P1–P7) | 七原則證據鏈證書 | 機內(模型↔真巨集雙通道) | 14/14 門禁 PASS;μ 遞減;Θ(n²) 比例 3.45 | ✅ CI Gate 10 |
| 14 | `borrow_model`(B1–B6) | 借用組合六門禁證書 | 機內(naive≡sweep≡Datalog 三算法差分+rep_dd 交叉驗證) | 24 組種子等價;κ-矩陣 3 格;層狀 depth×n | ✅ CI Gate 10 |
| 15 | `verify_all` / `ci_verify` | 端到端門禁總證書(10+7 門禁) | 機內+外部混合 | 8/10 Proven+2 SKIPPED(如實);ci_verify 7/7 | ✅ CI 綠 |

### 登記冊附註(冊法)

1. **三態誠實原則**:任何門禁結論只能是 Proven / Skipped / Failed;外部工具缺席 ⇒ Skipped 連原因印出,與 Proven 在輸出、計數、語義嚴格分離(F-01)。
2. **嚴格發布模式**:`verify_all --strict` / `CL0R0_STRICT=1` 下任一 Skipped 即非零退出——未裝 Rocq/Why3/Z3 的機器拿不到發布綠燈。
3. **證書可複現**:全部機內證書由 CI 常量驅動渲染,橫幅=編號=結論行,口徑永不漂移(F-06)。

---

## 表 3 · 功能歸類(八大類)

| 類別 | 職責 | 功能(表1 #) | 小計 |
|------|------|--------------|------|
| **一·詞法與語法層**(載體前置) | 字節流→token→樹,無損與增量 | lex(1)、parse(2)、span(3)、edit(4)、span_monad(5)、ast(6)、diff_tree(7)、tree(8) | 8 |
| **二·巨集層**(語法擴展) | 巨集語義之可執行模型與真巨集 | token_tree(9)、macro_lab(10)、bin macro_lab(52) | 3 |
| **三·借用與類型語義層** | 借用組合、契約、變異性 | borrow_model(11)、mir(19)、modular_contracts(20)、variance_dropck_ub(21)、polonius_bridge(22) | 5 |
| **四·重寫與合流層**(語義核心) | 終止、合流、正規形 | rep(12)、rep_dd(13)、dd_checker(14)、rule_labeling(15)、l9newman(16)、bin dd_verify(53)、bin l9newman(54)、maude_engine(35) | 8 |
| **五·證書與證明層**(形式化背書) | 證書載體、引理、外部證明器 | lemmas(23)、cpf_cert(24)、ari_export(25)、isabelle_export(26)、rocq_export(27)、creusot_export(28)、proof_resources(29)、tactics(30)、bin rocq_verify(55)、bin creusot_verify(56)、bin cert_factory(57)、bin dev_prover(63) | 12 |
| **六·實用載體與降階層** | R₀ 子集與語義降階 | r0(17)、r0_lower(18)、rustc_json(42) | 3 |
| **七·演算法基建層** | 合一、共享、索引 | unification(32)、dag_term(33)、discrimination_tree(34) | 3 |
| **八·測試、差分與工程運維層** | 屬性測試、差分、診斷、流水線 | gen(36)、shrink(37)、differential_checker(38)、lemma_stress_generator(39)、json_report(40)、lsp_bridge(41)、tool_runner(43)、patch_engine(44)、pipeline_synthesis(45)、cert_generator_factory(46)、testkit(47)、reparse_verifier(48)、tactic_scheduler(31)、bin cl0r0(49)、bin verify_all(50)、bin ci_verify(51)、bin coco_benchmark(58)、bin fuzz(59)、bin fuzz_daemon(60)、bin lemma_stress_coverage(61)、bin pipeline_runner(62)、bin lsp_server(64)、bin dev_loop(65) | 23 |

**分布**:語法/語義核心(類一、三、四)= 21 項 · 形式化背書(類五)= 12 項 · 品質工程(類八)= 22 項 ——「核心數學」與「驗證工程」雙軌並重。

---

## 附錄 · 本輪全碼審計發現與處置

### 已修(本輪)

| # | 發現 | 處置 |
|---|------|------|
| F-1 | CONTRIBUTING L48「96/96 項測試」過期 | → 123/123(以實測數為準) |
| F-2 | ci.yml 覆蓋率錨註解停留 68.04%(舊錨) | → 71.3%(lcov 逐檔加總,run 33961021652);門檻 65% 不變 |
| F-3 | README 缺全功能圖鑑導航 | → 增 FEATURE_ATLAS 連結 |
| F-4 | ci.yml 步驟名含未引號冒號致 YAML 解析失敗(前輪已修) | 引號化;登記備查 |
| F-5 | cargo-llvm-cov 舊旗標 `--lcov-output-path`(前輪已修) | → `--lcov --output-path`;登記備查 |

### 登記為技術債(未修,有後續建議)

| # | 債項 | 風險 | 建議 |
|---|------|------|------|
| D-1 | ~~覆蓋率冷點~~ | **已清(DL-001)**:五檔 91–98%,+34 測試;並修得兩處潛伏 bug(extract 作用域彈空 ⇒ 0 事件;借鏈不 walk ⇒ &mut 事件缺席) | — |
| D-2 | 16 個 bin 於 llvm-cov 顯示 0%(量測空洞) | bin 內邏輯無單測保護 | 將 bin 主體邏輯下沉為 lib 函數;或 `cargo llvm-cov --run-bin` |
| D-3 | 裸 unwrap 訊息化 | **第一批已清(DL-002)**:ari_export(11)+maude_engine(10)→ 0;餘量(lex 1、parse 6 等)留待第二批 | 後續批次:parse/ast/polonius_bridge/lemmas |
| D-4 | 覆蓋率門檻階梯 | **65→72 已清(DL-003,CI 實測 75.4%)**;下一級 80 | DL-004 下沉後再攻 80 |
| D-5 | `$k:path` 片段不可作巨集呼叫名(Rust 語言限制) | cl0_produce 需 `$k:ident` + 呼叫端解析 | 已定案 ident 方案;如需 path 級別,改 `macro_rules` 內 `use` 絕對路徑再呼叫 |
| D-6 | Rocq 9.2 不在 CI 環境(Gate 8 永遠 SKIPPED) | 形式化背書鏈缺一角 | CI 加 `rocq-overlay` 或容器側裝 Rocq;或標記 release 前置檢查清單 |
