# QUALITY_EVIDENCE — 質量証據匯編(DL-012)

> 機密 · © 2026 Ken Yuen Ka Chun。數據對應 CI run 34008381873(2026-09-06);
> 覆蓋率詳報見 `COVERAGE_REPORT.md`,此處為 data room 匯編口徑。

## 1 · 核心質量數字(全部機械可複核)

| 門禁 | 結果 | 複核命令 |
|------|------|----------|
| 單元/整合測試 | **200/200 通過** | `cargo test --workspace --all-targets` |
| 行覆蓋率 | **84.62%**(門檻 72) | CI `cargo llvm-cov --workspace --fail-under-lines 72` |
| clippy | **0 warning** | `cargo clippy --workspace --all-targets -- -D warnings` |
| rustfmt | 通過 | `cargo fmt --all -- --check` |
| 看板不變式 | **0 違規** | `cargo run --bin dev_loop` |
| 端到端自証 | **10/10 門禁 Proven · 0 SKIPPED · 0 FAILED** | `cargo run --bin verify_all -- --strict`(CI 環境) |
| 巨集門禁 | 14/14 PASS(P1–P7+B1–B6+Θ(n²)) | `cargo run --bin macro_lab` |
| 屬性套件 | 總失敗數 0(種子確定性) | `cargo run --bin fuzz` |
| 海量壓測 | 79,000 樣本,不變量 100% 保持 | `cargo run --bin lemma_stress_coverage` |
| 髒輸入穩健性 | 污料宇宙全數如實處置,0 panic | `cargo run --bin cert_factory` |

## 2 · 三態誠實語義(F-01)——質量體系地基

`Proven / Skipped / Failed` 全線統一:
- 外部證明器缺席 ⇒ **Skipped,絕不冒充 PASSED**;
- `--strict` 發布模式下 Skipped 即非零退出(發布阻斷);
- 歷史實績:曾經 9 Proven + 1 SKIPPED(Rocq 缺席),如實申報多輪,
  及後 CI 實裝 Rocq 9.2 方轉 10/10——**先誠實、後補証**嘅可審計軌跡。

## 3 · 過程中捉到並修復嘅潛伏 bug(測試文化实效証據)

| 時點 | Bug | 後果(若未發現) | 發現途徑 |
|------|-----|----------------|----------|
| DL-001 | extract 作用域棧彈空 ⇒ 0 事件 | 借用分析靜默失效 | 補測覆蓋時 |
| DL-001 | 借鏈分支不 walk ⇒ &mut 事件缺席 | 借用衝突漏報 | 補測覆蓋時 |
| DL-008 | 跨綁定衝突修剪:事件全部投影 storage 0+全域索引取區間 | 衝突修剪唔掉,bin 靜默印 ✗ | 下沉函數化+normalize_plan 測試 |

> 三者皆「綠燈下嘅暗病」——由覆蓋率提升與函數下沉工程揭出,非用戶報障。
> 此為「測試資產係捉 bug 機器而非指標裝飾」嘅直接証據。

## 4 · 覆蓋率分佈(逐檔要點)

- 七個曾 <74% 檔已全數達標:isabelle_export **100**、proof_resources **100**、
  variance_dropck_ub **98.4**、lsp_bridge **99.3**、reparse_verifier **88.0**、
  lemmas **81.9**、rocq_export **76.6**(CI lcov 口徑);
- 全庫最低檔 ≥72 門檻之上;無 0% lib 模組;
- 17 個 bin 顯示 0% 屬**量測空洞**(llvm-cov 只計儀器化測試;CI 每輪實跑
  全部 bin 且以退出碼把關)——DL-014 議程補真實量測。

## 5 · 質量不變式(寫入 CI,違者即斷)

1. 覆蓋率 < 72 ⇒ CI 失敗;
2. clippy 任何 warning ⇒ CI 失敗(`-D warnings`);
3. fmt 不齊 ⇒ CI 失敗;
4. 看板違規(WIP>2/done 無証據/非法隊)⇒ CI 失敗;
5. verify_all 於 CI(配齊 Rocq/Why3/Z3)必須 10/10 Proven。

> 以上五條令「綠色 CI」本身成為可轉讓嘅質量証據,而非自稱。
