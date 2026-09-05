//! §6.1 导出 CoCo 2025/2026 标准 ARI (Automated Rewriting Interface) 格式规范。

use std::fmt::Write;

pub fn export_menu_to_ari() -> String {
    let mut out = String::new();
    writeln!(
        out,
        ";; =========================================================="
    )
    .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
    writeln!(
        out,
        ";; CL0 Dual-Carrier Repair System (CoCo 2025/2026 ARI TRS)"
    )
    .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
    writeln!(
        out,
        ";; =========================================================="
    )
    .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
    writeln!(out, "(format trs)").expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
    writeln!(out, "(fun conf 3)").expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
    writeln!(out, "(fun pair 2)").expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
    writeln!(out, "(fun ok 1)").expect("不變式:寫入 String 緩衝,fmt::Error 不可能");

    writeln!(out, ";; Rule 1: Commutative Trim")
        .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
    writeln!(
        out,
        "(rule (pair (conf S A B) (conf S C D)) (pair (conf S A C) (conf S C D)))"
    )
    .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");

    writeln!(out, ";; Rule 4: Runtime Quarantine")
        .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
    writeln!(
        out,
        "(rule (pair (conf S A B) (conf S C D)) (ok (conf S A B)))"
    )
    .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");

    out
}
