//! cl0r0 —— 雙載體(CL0 定律載體 + R₀ 實用載體)的機械自証代碼庫。
//!
//! 六層基礎 → 九條定律(L1–L9) → 雙載體規格(§7) → 遞減圖/Newman 雙通道合流性自証:
//!   * `span`                  : σ(v) = [a,b) 半開區間(§1.2)
//!   * `lex`                   : CL0 詞法器 —— DFA,平鋪全源碼,trivia 保留(§5.1)
//!   * `parse`                 : 表面語法樹 + 增量重析 + ERROR 全化(§1–§2.3)
//!   * `tree`                  : 樹的性質檢查(連續性公理、laminar、CW 復形)
//!   * `edit`                  : 編輯單體(§2.1:位移函數、複合、結合律)
//!   * `ast`                   : 語義面 —— liveness 三軌(§3.2)、衝突圖(§3.3)
//!   * `rep`                   : 修法菜單重寫系統(§4:L8 遞減測度、L9 合流)
//!   * `rep_dd`                : van Oostrom 遞減圖 (Decreasing Diagrams) ARS 模組
//!   * `dd_checker`            : 遞減圖/Newman 雙模式局部峰值機械核驗器
//!   * `span_monad`            : AST 節點與事實層區間的雙向單子逆映射 (保持 L1/L6)
//!   * `patch_engine`          : 端到端補丁合成與閉環自驗引擎
//!   * `ari_export`            : CoCo 2025/2026 標準 ARI 格式導出器
//!   * `cpf_cert`              : 原生輕量級 CPF 證書生成與核驗器 (支持 KB 短證與 DD 偏序證)
//!   * `tactic_scheduler`      : 重寫策略調度器與 ARI-COPS 競賽對拍層
//!   * `pipeline_synthesis`    : 五大核心組合深度合成引擎 (1+2+3+4+5 閉環)
//!   * `cert_generator_factory`: 污料生成宇宙、形式化證書工廠與認證流水線
//!   * `shrink`                : 不變量驅動的語法樹 Delta 剪枝縮減算法
//!   * `r0`                    : R₀ Rust 子集 —— 附錄 B 覆蓋面契約 + unsupported 申報
//!   * `r0_lower`              : R₀ 實用載體語義降階與 Def-Use 活躍期分析
//!   * `reparse_verifier`      : L3/L4 增量重析等價性核驗器
//!   * `rustc_json`            : 強制 .json 格式報錯解析與自動機套接
//!   * `polonius_bridge`       : Polonius 官方 Datalog 關係事實雙向橋接
//!   * `lsp_bridge`            : Language Server Protocol 交互式修法橋接 (附 Newman/DD 解釋)
//!   * `gen`                   : 合法程式 + 髒輸入生成器
//!   * `l9newman`              : 機械的 Newman 通道
//!   * `mir`                   : MIR 控制流圖、Move 分析、Drop 展開與 NLL 借用檢查器 (§8.1)
//!   * `modular_contracts`     : OOPSLA 2025 模組化函數契約、Reborrow 懸掛與循環不動點求解器 (§8.2)
//!   * `proof_resources`       : Aeneas 反向函數、Creusot 預言模型與 Prusti 分離邏輯 (§8.3)
//!   * `variance_dropck_ub`    : 型別變異性 (Variance)、Dropck 針眼法則與 UB 診斷預言機 (§8.4)
//!   * `diff_tree`             : 持久化結構共享 AST 與狀態差分增量引擎 (§9.1)
//!   * `differential_checker`  : 跨引擎語意差分測試、黃金對賬與 DDMin 自動收斂器 (§9.2)

pub mod ari_export;
pub mod ast;
pub mod cert_generator_factory;
pub mod cpf_cert;
pub mod dag_term;
pub mod dd_checker;
pub mod diff_tree;
pub mod differential_checker;
pub mod discrimination_tree;
pub mod edit;
pub mod gen;
pub mod isabelle_export;
pub mod json_report;
pub mod l9newman;
pub mod lemma_stress_generator;
pub mod lemmas;
pub mod lex;
pub mod lsp_bridge;
pub mod maude_engine;
pub mod mir;
pub mod modular_contracts;
pub mod parse;
pub mod patch_engine;
pub mod pipeline_synthesis;
pub mod polonius_bridge;
pub mod proof_resources;
pub mod r0;
pub mod r0_lower;
pub mod rep;
pub mod rep_dd;
pub mod reparse_verifier;
pub mod rule_labeling;
pub mod rustc_json;
pub mod shrink;
pub mod span;
pub mod span_monad;
pub mod tactic_scheduler;
pub mod tactics;
pub mod tree;
pub mod unification;
pub mod variance_dropck_ub;
