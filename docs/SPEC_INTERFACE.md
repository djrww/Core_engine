# SPEC_INTERFACE — 介面規格(DL-012)

> 機密 · © 2026 Ken Yuen Ka Chun. 對應版本:v0.2.0 / 2026-09-06。

## 1 · 對外介面總覽(三個)

| 介面 | 協議 | 消費方 | 穩定性 |
|---|---|---|---|
| **LSP** | JSON-RPC 2.0(Language Server Protocol 標準) | IDE 插件(VS Code/JetBrains/Neovim/Zed) | 穩定;產品化主通道 |
| **CLI** | stdout/退出碼 | 開發者、CI | 穩定;自証與量測用 |
| **lib API** | `cl0r0::X` 單一命名空間 | 內部與嵌入場景 | 隨版本演進 |

## 2 · LSP 介面(產品化主通道)

方法表(`lsp_bridge::LspEngine::process_json_rpc`,JSON-RPC 2.0):

| Method | 行為 | 回應特徵 |
|---|---|---|
| `initialize` | 引擎能力宣示 | 含 `"id"` 回顯 |
| `textDocument/codeAction` | 借用衝突診斷 + **合流修補 quickfix** | `edit_replacement`(修補後源碼)+ tactic 標記 |
| `textDocument/hover` | 定點語義解釋 | markdown |
| `textDocument/inlayHint` | 行內證據提示 | `[✓ DD Confluent]` 標記 |
| `textDocument/formatting` | 格式化(現階段空清單,如實申報) | `"result":[]` |
| `shutdown` | 關閉 | `"result":null` |
| (其他/未知) | 標準 JSON-RPC 錯誤 | method not found |

**診斷語義**:E0502 類借用衝突,附 `proof_explanation`(Newman 快速通道或
DD 遞減圖解釋,按衝突規模分流);quickfix 為合流修補(縮短借用區域),
修補經 L3/L4 增量重析等價自驗後方出示——**不出示未經自驗的建議**。

**目標部署形態**:單一 `.wasi`/原生二進位承載 LSP server(`cl0r0_lsp`),
IDE 插件只做薄殼。跨平台發布決策見 BACKLOG DL-011(parked,待 beta 鎖版)。

## 3 · CLI 介面(17 個自証 bin)

| bin | 用途 | 退出碼語義 |
|---|---|---|
| `cl0r0` | 主演示:九律檢查+幾何/重寫演示 | 0=演示全程一致 |
| `verify_all` | 端到端 10 門禁(支持 `--strict` 發布模式) | strict 下 SKIPPED 即非零(F-01) |
| `ci_verify` | CI 專項 7 門禁 | 同上 |
| `dev_loop` | 看板不變式裁判(WIP≤2/done 必有證據/隊合法) | 違規即非零 |
| `macro_lab` | 巨集七原則 P1–P7+借用組合 B1–B6(14 門禁) | 0=14/14 PASS |
| `l9newman` | Newman 快速通道獨立驅動(SN 見證出具) | 0=SN∧WBR⇒CR 成立 |
| `dd_verify` | 遞減圖+Newman 自証驅動(CPF-KB 短證出具) | 0=峰值會合 |
| `rocq_verify` | Rocq 導出+rocqchk 微內核核檢 | 三態(Proven/Skipped/Failed) |
| `creusot_verify` | Creusot/Why3/Z3 VC 消解 | 三態 |
| `fuzz` / `fuzz_daemon` | 種子確定性屬性套件/定時守護 | 0=總失敗數 0 |
| `lemma_stress_coverage` | 79k 樣本海量壓測+覆蓋率 | 0=不變量 100% 保持 |
| `cert_factory` | 污料宇宙+證書批量生產(穩健性) | 0=髒輸入全數如實處置 |
| `pipeline_runner` | 五大組合深度合成驗證 | 0=五階段閉環 |
| `coco_benchmark` | 國際合流基準(CoCo)對接位 | 0=基準跑畢 |
| `cl0r0_lsp` | 獨立 LSP server(JSON-RPC stdin/stdout 循環) | 服務語義 |
| `dev_prover` | 開發階段引理取証+証物打包 | 0=証物齊 |

**三態誠實語義(F-01,全線統一)**:`Proven / Skipped / Failed`;
外部證明器缺席 ⇒ **Skipped(絕不冒充 PASSED)**;`--strict` 發布模式下游
Skipped 即阻斷。此語義為本引擎與「全綠幻覺」工具嘅根本分別。

## 4 · lib API 原則

- 單一命名空間 `cl0r0::X`(facade re-export;內部五 crate 重組不影響外部);
- 類型即証據:`CPFCertificate`/`LemmaVerificationResult`/`GateStatus` 等,
  結論與見證同構攜帶,拒絕「裸結論」;
- 錯誤處理:非測試碼裸 unwrap = 0(全數 `expect` 帶不變式訊息)。
