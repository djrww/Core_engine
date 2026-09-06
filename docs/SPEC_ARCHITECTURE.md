# SPEC_ARCHITECTURE — 架構規格(DL-012)

> 機密 · © 2026 Ken Yuen Ka Chun. 對應版本:v0.2.0 / Atlas v0.3.0 / 2026-09-06。
> 全景圖與設計規範詳見根目錄 `ARCHITECTURE.md`;本檔為 data room 摘要規格。

## 1 · 架構總覽

Cargo workspace 五 crate,依賴方向**單向、由編譯器強制**(不允許環/上跳):

```
┌─────────────────────────────────────────────────┐
│  C5  cl0r0 (facade)                              │
│      10 lib 模組 + 17 自証 bin                    │
│      lemmas/selfcheck/lsp_bridge/fuzz_engine/…   │
├─────────────────────────────────────────────────┤
│  C4  cl0r0-cert        證書與外部證明層(6 模組)   │
│      CPF 消費/Rocq/Isabelle/Ari/Creusot 導出      │
├─────────────────────────────────────────────────┤
│  C3  cl0r0-ars         重寫/合流核心(22 模組)     │
│      ARS/DD/Newman/KB/合一/判別樹/Polonius/巨集    │
├─────────────────────────────────────────────────┤
│  C2  cl0r0-mir         中層 IR(3 模組)           │
│      MIR 類型/模組化契約/變異性·dropck·UB 預言機   │
├─────────────────────────────────────────────────┤
│  C1  cl0r0-syntax      語法層(9 模組)            │
│      lex/parse/ast/token_tree/edit/diff_tree     │
└─────────────────────────────────────────────────┘
```

## 2 · 分層語義(點解咁分)

| 層 | 數學職責 | 對應定理/規律 |
|---|---|---|
| C1 語法 | 字節流 ⇄ 樹,無損回環、增量重析 | L1 無損回環、L2 決定論、L3/L4 增量等價、L7 錯誤全量化 |
| C2 MIR | 類型/區域/變異性中間表示 | 變異格(lattice)、dropck 針眼法則、UB 邊界 |
| C3 合流 | 抽象重寫系統:衝突 ⇔ 臨界對 ⇔ 會合 | L5 層流、L8 良基遞減、L9 遞減圖、Newman、KB、L11 區間圖弦性 |
| C4 證書 | 証據出具與外部核檢 | CPF 證書、Rocq 微內核、Creusot VC、Isabelle 草案 |
| C5 門面 | 對外單一真相 + 工程組合 | L6 命名投影同態、18 引理矩陣、LSP |

**設計不變式**:
1. 對外 API 只有 `cl0r0::X` 一個命名空間(C5 re-export),內部重組外界零感知;
2. 每個 crate 可獨立測試(測試隨模組走);
3. 第三方運行時依賴 = **0**(僅 Rust std)。

## 3 · 模組 → crate 映射(51 lib 模組全表)

| crate | 模組(數) |
|---|---|
| C1 cl0r0-syntax(9) | span, lex, edit, gen, ast, parse, tree, token_tree, diff_tree |
| C2 cl0r0-mir(3) | mir, modular_contracts, variance_dropck_ub |
| C3 cl0r0-ars(22) | cpf_cert, dag_term, unification, discrimination_tree, tactics, maude_engine, rep, rep_dd, dd_checker, rule_labeling, tactic_scheduler, l9newman, span_monad, shrink, reparse_verifier, r0, r0_lower, borrow_model, patch_engine, polonius_bridge, macro_lab, testkit |
| C4 cl0r0-cert(6) | tool_runner, rocq_export, creusot_export, isabelle_export, ari_export, proof_resources |
| C5 cl0r0 facade(10) | lemmas, selfcheck, json_report, rustc_json, lsp_bridge, fuzz_engine, differential_checker, lemma_stress_generator, cert_generator_factory, pipeline_synthesis |

> 粘連點決策紀錄(點解 cpf_cert 喺 C3、patch_engine 喺 C3 等)見 `WORKSPACE_PLAN.md` §3。

## 4 · 關鍵數字

| 指標 | 值 |
|---|---|
| workspace 行覆蓋率 | **90.87%**(CI lcov run 34027780280,門檻 72) |
| 測試 | **218/218**(facade 141 + 子 crate 74 + bins/integration 60,含 bins_smoke 17) |
| clippy | 0 warning(`-D warnings` 標準) |
| 裸 unwrap(非測試碼) | 0 |
| lib 模組/bin | 51 / 17 |
| 語義模型 | R₀ 刻意收窄子集(誠實申報,見 SPEC_CERTIFICATES §4) |
