# WORKSPACE_PLAN — 5-crate 拆分計畫(DL-010)

> 狀態:**已獲產品負責人批准並執行完畢**(2026-09-06;C1..C5 全數遷移,workspace 200 測試綠)。
> 原則:①依賴方向由編譯器強制,拒絕環;②對外 API(`cl0r0::` 路徑)零變化;
> ③測試隨模組走;④CI/覆蓋率口徑明確;⑤每步可回退(獨立 commit)。

## 1 · 目標結構

```
core_engine/                    (Cargo workspace)
├── crates/
│   ├── cl0r0-syntax/   C1 語法層   9 模組
│   ├── cl0r0-mir/      C2 MIR 層   3 模組
│   ├── cl0r0-ars/      C3 重寫合流層 22 模組
│   └── cl0r0-cert/     C4 證書導出層 6 模組
├── src/                C5 facade crate `cl0r0`(lib + 17 bins,名不變)
├── Cargo.toml          [workspace]
└── .github/workflows/ci.yml
```

依賴方向(只能向下):**C5 → C4 → C3 → C2 → C1**;嚴禁反向與跨層上跳。

## 2 · 模組映射表(51 lib 模組逐個歸層)

### C1 `cl0r0-syntax`(9)
span, lex, edit, gen, ast, parse, tree, token_tree, diff_tree
— 內部依賴:parse→edit/lex/span;tree→parse;diff_tree→span;gen→edit。全內部 ✓

### C2 `cl0r0-mir`(3)
mir, modular_contracts, variance_dropck_ub
— mir→span(C1)✓;modular_contracts→mir;variance_dropck_ub→mir。

### C3 `cl0r0-ars` 重寫/合流核心(22)
cpf_cert(零依賴類型層), dag_term, unification, discrimination_tree, tactics,
maude_engine, rep, rep_dd, dd_checker, rule_labeling, tactic_scheduler,
l9newman, span_monad, shrink, reparse_verifier, r0, r0_lower, borrow_model,
patch_engine, polonius_bridge, macro_lab, testkit
— 對下:rep→ast(C1);polonius_bridge→ast/parse/patch_engine(同層);
  reparse_verifier→edit/parse(C1);borrow_model→ast/gen/rep_dd/macro_lab/testkit(同層)✓

### C4 `cl0r0-cert`(6)
tool_runner, rocq_export, creusot_export, isabelle_export, proof_resources, ari_export
— rocq_export→cpf_cert(C3)+tool_runner(同層);proof_resources→mir(C2)✓

### C5 `cl0r0` facade(10 lib 模組 + 17 bins)
lemmas, selfcheck, json_report, rustc_json, lsp_bridge, fuzz_engine,
differential_checker, lemma_stress_generator, cert_generator_factory, pipeline_synthesis + lib.rs(re-export 全部,保持 `cl0r0::X` 路徑不變)+ 17 bins
— 對下全為向下引用;rustc_json↔json_report 同層;無環 ✓

## 3 · 已知粘連點與對策

| # | 粘連 | 對策 |
|---|------|------|
| K1 | borrow_model 用 macro_lab 嘅 `macro_rules!` 巨集(cl0_with_val 等) | 兩者同置 C3;巨集以 `#[macro_use]` 或 pub use 導出,路徑不變 |
| K2 | cpf_cert 被 C3(dd_checker)與 C4(exporters)共用 | cpf_cert 屬 C3(證書類型是合流核心輸出),C4 向下引用 ✓ |
| K3 | patch_engine 被 polonius_bridge(C3)與 lsp_bridge(C5)共用 | patch_engine 歸 C3(只依賴 C1,補丁合成是合流工程核心);C5 向下 ✓ |
| K4 | testkit 是測試夾具但被 C3/C5 多模組用 | 歸 C3 尾層,僅 `#[cfg(test)]`+pub 導出;或者遷移時改名 cl0r0-ars::testkit |
| K5 | `crate::X` 路徑 51 檔全改 | 機械替換腳本:`crate::X` → `cl0r0_X::X`(C1-C4 各自內部保留 crate::) |

## 4 · 覆蓋率/CI 口徑

- `cargo llvm-cov --workspace`:門檻維持 **全庫合併 ≥72**(--fail-under-lines 不變);
- 新增每 crate 明細報表(僅展示,不設 per-crate 門檻,避免首次拆分即卡);
- CI 步驟不改(命令 `--workspace` 自動覆蓋);bins 編譯命令不變;
- dev_loop/verify_all 等自証輸出應零變化(純內部重組)。

## 5 · 遷移步驟(每步一個 commit,可獨立回退)

1. 建 workspace 骨架:根 Cargo.toml `[workspace]`,crates/ 四目錄,空 lib
2. C1 起底:搬 9 模組,改 use,`cargo test -p cl0r0-syntax` 綠
3. C2 → C3(最大批,可拆兩片:C3a 純重寫、C3b 工程+巨集)→ C4
4. C5 facade:lib.rs re-export;51 個 `crate::` 路徑機械替換;200 測試全綠
5. bins 驗證:17 個 bin 實跑(verify_all 10/10 等)
6. CI 綠 → Atlas v0.3 表1 增「crate 歸屬」欄 → 文檔同步
7. 看板 DL-010 done 入冊

## 6 · 驗收門(照 BACKLOG)

依賴方向編譯器強制(無環);200 測試全綠;17 bin 實跑不變;CI 綠;Atlas 同步。

## 7 · 風險

- macro_rules 跨 crate 可見性(K1)——最可能出意外處,優先做 spike 驗證;
- 覆蓋率歸檔路徑變化(llvm-cov 產物路徑)——CI artifact 收集路徑要跟;
- 編譯時間:5 crate 增量編譯應更快,但首次全量略增(可接受)。
