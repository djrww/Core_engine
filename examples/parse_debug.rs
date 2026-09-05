//! parse_debug —— 解析器 AST 傾印調試腳手架(原倉庫根 scratch.rs,審計 F-10 遷移)。
//!
//! 過去倉庫根的 scratch.rs 不在任何 crate target 內,CI 不編譯它,
//! 只會靜靜腐爛;遷入 examples/ 後納入 `cargo build --examples` 與
//! clippy --all-targets 門禁,腳手架有了生命週期。
//!
//! 運行: `cargo run --example parse_debug`

use cl0r0::parse::parse;

fn main() {
    let cut = "fn g() {\n  while &mut g(&z == k()) < 21 {\n  if trulet \n  let w;\n}\n}\n";
    let t = parse(cut).expect("debug fixture must parse (ERROR 全化)");
    for (i, n) in t.nodes.iter().enumerate() {
        println!(
            "{:3} {:12} {:?} children={:?}",
            i,
            format!("{:?}", n.kind),
            n.span,
            n.children
        );
    }
}
