//! `cl0r0-ars` —— 重寫/合流核心(C3)
//!
//! 抽象重寫系統(ARS)、DD/Newman/KB 三通道、合一/判別樹/索引、
//! Polonius 借用橋、巨集實驗室(`borrow_model`↔`macro_lab` 同 crate 消解環)。
//! 依賴方向:僅向下依賴 C1(cl0r0-syntax)。

// 令 crate::X 於本 crate 內照常解析(C1 依賴透傳)
pub use cl0r0_syntax::{ast, edit, gen, lex, parse, span, token_tree};

pub mod borrow_model;
pub mod cpf_cert;
pub mod cpf_import;
pub mod dag_term;
pub mod dd_checker;
pub mod discrimination_tree;
pub mod l9newman;
pub mod macro_lab;
pub mod maude_engine;
pub mod patch_engine;
pub mod polonius_bridge;
pub mod r0;
pub mod r0_lower;
pub mod rep;
pub mod rep_dd;
pub mod reparse_verifier;
pub mod rule_labeling;
pub mod shrink;
pub mod span_monad;
pub mod tactic_scheduler;
pub mod tactics;
pub mod testkit;
pub mod unification;
