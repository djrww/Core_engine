//! §6.1 导出 CoCo 2025/2026 标准 ARI (Automated Rewriting Interface) 格式规范。

use std::fmt::Write;

pub fn export_menu_to_ari() -> String {
    let mut out = String::new();
    writeln!(
        out,
        ";; =========================================================="
    )
    .unwrap();
    writeln!(
        out,
        ";; CL0 Dual-Carrier Repair System (CoCo 2025/2026 ARI TRS)"
    )
    .unwrap();
    writeln!(
        out,
        ";; =========================================================="
    )
    .unwrap();
    writeln!(out, "(format trs)").unwrap();
    writeln!(out, "(fun conf 3)").unwrap();
    writeln!(out, "(fun pair 2)").unwrap();
    writeln!(out, "(fun ok 1)").unwrap();

    writeln!(out, ";; Rule 1: Commutative Trim").unwrap();
    writeln!(
        out,
        "(rule (pair (conf S A B) (conf S C D)) (pair (conf S A C) (conf S C D)))"
    )
    .unwrap();

    writeln!(out, ";; Rule 4: Runtime Quarantine").unwrap();
    writeln!(
        out,
        "(rule (pair (conf S A B) (conf S C D)) (ok (conf S A B)))"
    )
    .unwrap();

    out
}
