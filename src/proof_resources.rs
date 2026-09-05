//! §8.3 Rust 類型作為證明資源：Aeneas 反向函數、Creusot 預言模型與 Prusti 分離邏輯 (Proof Resources Engine)。
//!
//! 將 Rust 嚴格的 Ownership / Borrowing 語義轉化為高階形式化證明資源：
//!   1. **Aeneas (Charon) 函數式轉換 (Backward Functions)**：
//!      - 將含 `&mut T` 的命令式代碼無損轉化為純函數對：
//!        - 正向函數 $f_{\text{fwd}}(x) \to y$
//!        - 反向函數 $f_{\text{back}}(x, y) \to x'$ (返還可變更新後的數值)
//!      - 證明借用檢查器的無別名定理保證了反向函數轉換的語義等價性與確定性。
//!   2. **Creusot (Pearlite) 預言變量演算 (Prophecy Calculus)**：
//!      - 將可變引用 `&'a mut T` 建模為序對 $(\text{val}_{\text{cur}}, \text{val}_{\text{prophecy}})$。
//!      - 當借用終止時，觸發預言 Resolve，將最終計算值回寫至原主。
//!   3. **Prusti / Viper 分離邏輯分數權限 (Fractional Permissions)**：
//!      - 共享引用 `&T` $\implies \text{Perm::Shared}(q), q \in (0, 1]$
//!      - 獨佔引用 `&mut T` / 擁有權 $T \implies \text{Perm::Exclusive}(1.0)$
//!      - 權限代數保證任意位置處 $\sum q \le 1.0$。

use crate::mir::MirType;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

// =========================================================================
// 1. Aeneas 純函數式轉換與 Backward Functions
// =========================================================================

/// 純函數式 AST 表達式
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PureExpr {
    Var(String),
    ConstInt(i64),
    Tuple(Vec<PureExpr>),
    Apply(String, Vec<PureExpr>),
    Let(String, Box<PureExpr>, Box<PureExpr>),
    Add(Box<PureExpr>, Box<PureExpr>),
    Sub(Box<PureExpr>, Box<PureExpr>),
    Mul(Box<PureExpr>, Box<PureExpr>),
}

impl Display for PureExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PureExpr::Var(s) => write!(f, "{}", s),
            PureExpr::ConstInt(n) => write!(f, "{}", n),
            PureExpr::Tuple(es) => {
                write!(f, "(")?;
                for (i, e) in es.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            PureExpr::Apply(name, args) => {
                write!(f, "{}(", name)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ")")
            }
            PureExpr::Let(var, val, body) => write!(f, "let {} = {} in {}", var, val, body),
            PureExpr::Add(a, b) => write!(f, "({} + {})", a, b),
            PureExpr::Sub(a, b) => write!(f, "({} - {})", a, b),
            PureExpr::Mul(a, b) => write!(f, "({} * {})", a, b),
        }
    }
}

/// Aeneas 轉換後的一組純函數 (正向 Forward + 各可變參數的反向 Backward 函數)
#[derive(Clone, Debug)]
pub struct AeneasTranslatedFunction {
    pub name: String,
    pub forward_params: Vec<(String, MirType)>,
    pub forward_body: PureExpr,
    pub backward_functions: Vec<(String, PureExpr)>, // (param_name, backward_expr)
}

pub struct AeneasTranslator;

impl AeneasTranslator {
    /// 對典型含 `&mut x` 的命令式代碼執行 Aeneas 函數式轉換
    /// 例如 `fn increment(x: &mut i32)` 轉化為：
    ///   - fwd: `increment_fwd(x) = ()`
    ///   - back: `increment_back(x) = x + 1`
    pub fn translate_increment_example() -> AeneasTranslatedFunction {
        AeneasTranslatedFunction {
            name: "increment".into(),
            forward_params: vec![("x".into(), MirType::Int(32))],
            forward_body: PureExpr::Tuple(vec![]),
            backward_functions: vec![(
                "x".into(),
                PureExpr::Add(
                    Box::new(PureExpr::Var("x".into())),
                    Box::new(PureExpr::ConstInt(1)),
                ),
            )],
        }
    }

    /// 典型交換函數 `fn swap(x: &mut i32, y: &mut i32)` 的 Aeneas 轉換
    ///   - fwd: `swap_fwd(x, y) = ()`
    ///   - back_x: `swap_back_x(x, y) = y`
    ///   - back_y: `swap_back_y(x, y) = x`
    pub fn translate_swap_example() -> AeneasTranslatedFunction {
        AeneasTranslatedFunction {
            name: "swap".into(),
            forward_params: vec![
                ("x".into(), MirType::Int(32)),
                ("y".into(), MirType::Int(32)),
            ],
            forward_body: PureExpr::Tuple(vec![]),
            backward_functions: vec![
                ("x".into(), PureExpr::Var("y".into())),
                ("y".into(), PureExpr::Var("x".into())),
            ],
        }
    }

    /// 執行純函數求值 (用於驗證反向函數轉換的確定性)
    pub fn eval_expr(expr: &PureExpr, env: &HashMap<String, i64>) -> i64 {
        match expr {
            PureExpr::Var(name) => *env.get(name).unwrap_or(&0),
            PureExpr::ConstInt(n) => *n,
            PureExpr::Tuple(_) => 0,
            PureExpr::Apply(_, _) => 0,
            PureExpr::Let(var, val, body) => {
                let v = Self::eval_expr(val, env);
                let mut new_env = env.clone();
                new_env.insert(var.clone(), v);
                Self::eval_expr(body, &new_env)
            }
            PureExpr::Add(a, b) => Self::eval_expr(a, env) + Self::eval_expr(b, env),
            PureExpr::Sub(a, b) => Self::eval_expr(a, env) - Self::eval_expr(b, env),
            PureExpr::Mul(a, b) => Self::eval_expr(a, env) * Self::eval_expr(b, env),
        }
    }
}

// =========================================================================
// 2. Creusot (Pearlite) 預言變量演算 (Prophecy Calculus)
// =========================================================================

/// 預言狀態對象 (Current Value, Prophecy Variable)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProphecyCell<T: Clone> {
    pub current_val: T,
    pub prophecy_val: T,
    pub is_resolved: bool,
}

impl<T: Clone> ProphecyCell<T> {
    pub fn new(initial: T, prophecy: T) -> Self {
        Self {
            current_val: initial,
            prophecy_val: prophecy,
            is_resolved: false,
        }
    }

    /// 觸發預言 Resolution: 將預言終值提交至當前狀態
    pub fn resolve(&mut self) -> T {
        self.current_val = self.prophecy_val.clone();
        self.is_resolved = true;
        self.current_val.clone()
    }
}

/// Creusot 預言環境與可變借用追蹤器 (Creusot Prophecy Tracker)
#[derive(Clone, Debug, Default)]
pub struct ProphecyEnvironment {
    pub cells: HashMap<String, ProphecyCell<i64>>,
    pub reborrow_parents: HashMap<String, String>, // child_borrow -> parent_borrow
}

impl ProphecyEnvironment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_borrow(&mut self, var_name: &str, current: i64, prophecy: i64) {
        self.cells
            .insert(var_name.to_string(), ProphecyCell::new(current, prophecy));
    }

    /// 註冊 Reborrow: 子借用繼承父借用的 current 值，並在結束時傳遞預言值
    pub fn register_reborrow(&mut self, parent_name: &str, child_name: &str) -> Result<(), String> {
        let parent_cell = self
            .cells
            .get(parent_name)
            .ok_or_else(|| format!("Parent borrow {} not found", parent_name))?
            .clone();

        // 子借用具有相同的初始 current 和預期的 prophecy
        self.cells.insert(
            child_name.to_string(),
            ProphecyCell::new(parent_cell.current_val, parent_cell.prophecy_val),
        );
        self.reborrow_parents
            .insert(child_name.to_string(), parent_name.to_string());
        Ok(())
    }

    pub fn resolve_borrow(&mut self, var_name: &str) -> Option<i64> {
        if let Some(cell) = self.cells.get_mut(var_name) {
            let final_val = cell.resolve();
            // 若為子借用，則自動更新父借用的 current 狀態
            if let Some(parent) = self.reborrow_parents.get(var_name).cloned() {
                if let Some(pcell) = self.cells.get_mut(&parent) {
                    pcell.current_val = final_val;
                }
            }
            Some(final_val)
        } else {
            None
        }
    }
}

// =========================================================================
// 3. Prusti 分離邏輯分數權限模型 (Fractional Permissions)
// =========================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Permission {
    None,
    Shared(f64), // 0.0 < q <= 1.0
    Exclusive,   // 1.0
}

impl Permission {
    pub fn is_readable(&self) -> bool {
        match self {
            Permission::None => false,
            Permission::Shared(q) => *q > 0.0,
            Permission::Exclusive => true,
        }
    }

    pub fn is_writable(&self) -> bool {
        matches!(self, Permission::Exclusive)
    }

    /// 權限拆分 (Split: Exclusive -> 2 * Shared(0.5))
    pub fn split(&self) -> Result<(Permission, Permission), String> {
        match self {
            Permission::Exclusive => Ok((Permission::Shared(0.5), Permission::Shared(0.5))),
            Permission::Shared(q) if *q > 1e-6 => {
                let half = *q / 2.0;
                Ok((Permission::Shared(half), Permission::Shared(half)))
            }
            _ => Err("無法拆分空權限".into()),
        }
    }

    /// 權限合併 (Join: Shared(q1) + Shared(q2))
    pub fn join(&self, other: &Permission) -> Result<Permission, String> {
        match (self, other) {
            (Permission::None, p) | (p, Permission::None) => Ok(*p),
            (Permission::Shared(q1), Permission::Shared(q2)) => {
                let sum = q1 + q2;
                if (sum - 1.0).abs() < 1e-6 {
                    Ok(Permission::Exclusive)
                } else if sum < 1.0 {
                    Ok(Permission::Shared(sum))
                } else {
                    Err(format!("權限總和 {} 超出 1.0 (違反分離邏輯守恆律)", sum))
                }
            }
            _ => Err("衝突的排他權限合併".into()),
        }
    }
}

/// Prusti 權限狀態記錄表
#[derive(Clone, Debug, Default)]
pub struct PermissionState {
    pub place_perms: HashMap<String, Permission>,
}

impl PermissionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_permission(&mut self, place: &str, perm: Permission) {
        self.place_perms.insert(place.to_string(), perm);
    }

    pub fn get_permission(&self, place: &str) -> Permission {
        self.place_perms
            .get(place)
            .copied()
            .unwrap_or(Permission::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aeneas_backward_functional_translation() {
        let inc_fn = AeneasTranslator::translate_increment_example();
        assert_eq!(inc_fn.name, "increment");
        assert_eq!(inc_fn.backward_functions.len(), 1);

        let mut env = HashMap::new();
        env.insert("x".into(), 41);

        let back_expr = &inc_fn.backward_functions[0].1;
        let final_x = AeneasTranslator::eval_expr(back_expr, &env);
        assert_eq!(final_x, 42);
    }

    #[test]
    fn test_aeneas_swap_translation() {
        let swap_fn = AeneasTranslator::translate_swap_example();
        let mut env = HashMap::new();
        env.insert("x".into(), 10);
        env.insert("y".into(), 20);

        let back_x = &swap_fn.backward_functions[0].1;
        let back_y = &swap_fn.backward_functions[1].1;

        assert_eq!(AeneasTranslator::eval_expr(back_x, &env), 20);
        assert_eq!(AeneasTranslator::eval_expr(back_y, &env), 10);
    }

    #[test]
    fn test_creusot_prophecy_resolution() {
        let mut penv = ProphecyEnvironment::new();
        // 初始值為 100，借用期間承諾將其改為 200
        penv.register_borrow("ptr", 100, 200);

        assert_eq!(penv.cells["ptr"].current_val, 100);
        assert!(!penv.cells["ptr"].is_resolved);

        let res = penv.resolve_borrow("ptr");
        assert_eq!(res, Some(200));
        assert_eq!(penv.cells["ptr"].current_val, 200);
        assert!(penv.cells["ptr"].is_resolved);
    }

    #[test]
    fn test_creusot_reborrow_prophecy_chain() {
        let mut penv = ProphecyEnvironment::new();
        penv.register_borrow("parent", 10, 50);
        assert!(penv.register_reborrow("parent", "child").is_ok());

        assert_eq!(penv.cells["child"].current_val, 10);
        assert_eq!(penv.cells["child"].prophecy_val, 50);

        let child_res = penv.resolve_borrow("child");
        assert_eq!(child_res, Some(50));
        assert_eq!(penv.cells["parent"].current_val, 50);
    }

    #[test]
    fn test_prusti_fractional_permissions() {
        let full = Permission::Exclusive;
        assert!(full.is_writable());
        assert!(full.is_readable());

        let (p1, p2) = full.split().unwrap();
        assert_eq!(p1, Permission::Shared(0.5));
        assert_eq!(p2, Permission::Shared(0.5));
        assert!(!p1.is_writable());
        assert!(p1.is_readable());

        let joined = p1.join(&p2).unwrap();
        assert_eq!(joined, Permission::Exclusive);
        assert!(joined.is_writable());
    }

    #[test]
    fn test_pure_expr_display_all_seven_forms() {
        use PureExpr::*;
        let v = Var("x".into());
        let c = ConstInt(7);
        let t = Tuple(vec![ConstInt(1), ConstInt(2), Var("y".into())]);
        let a = Apply("succ".into(), vec![ConstInt(41), ConstInt(42)]);
        let l = Let(
            "z".into(),
            Box::new(ConstInt(3)),
            Box::new(Add(Box::new(Var("z".into())), Box::new(ConstInt(1)))),
        );
        let add = Add(Box::new(ConstInt(1)), Box::new(ConstInt(2)));
        let sub = Sub(Box::new(ConstInt(5)), Box::new(ConstInt(2)));
        let mul = Mul(Box::new(ConstInt(3)), Box::new(ConstInt(4)));
        // 逐一驗證七種形式的 Display 語法
        assert_eq!(v.to_string(), "x");
        assert_eq!(c.to_string(), "7");
        assert_eq!(t.to_string(), "(1, 2, y)");
        assert_eq!(a.to_string(), "succ(41, 42)");
        assert_eq!(l.to_string(), "let z = 3 in (z + 1)");
        assert_eq!(add.to_string(), "(1 + 2)");
        assert_eq!(sub.to_string(), "(5 - 2)");
        assert_eq!(mul.to_string(), "(3 * 4)");
        // 嵌套組合亦應保持括號平衡
        let nested = Mul(Box::new(add), Box::new(sub));
        assert_eq!(nested.to_string(), "((1 + 2) * (5 - 2))");
    }

    #[test]
    fn test_eval_expr_let_scoping_and_arithmetic() {
        let mut env = HashMap::new();
        env.insert("base".to_string(), 10);
        // Let 綁定局部作用域:let z = 5 in (z - base)
        let expr = PureExpr::Let(
            "z".into(),
            Box::new(PureExpr::ConstInt(5)),
            Box::new(PureExpr::Sub(
                Box::new(PureExpr::Var("z".into())),
                Box::new(PureExpr::Var("base".into())),
            )),
        );
        assert_eq!(AeneasTranslator::eval_expr(&expr, &env), 5 - 10);
        // Var 未綁定 → 預設 0;Mul/Sub 實算。
        let mul = PureExpr::Mul(
            Box::new(PureExpr::Var("unknown".into())),
            Box::new(PureExpr::ConstInt(9)),
        );
        assert_eq!(AeneasTranslator::eval_expr(&mul, &HashMap::new()), 0);
        let sub = PureExpr::Sub(
            Box::new(PureExpr::ConstInt(2)),
            Box::new(PureExpr::ConstInt(9)),
        );
        assert_eq!(AeneasTranslator::eval_expr(&sub, &HashMap::new()), -7);
        // Tuple/Apply 求值恆為 0(純化約定的佔位值)。
        let tup = PureExpr::Tuple(vec![PureExpr::ConstInt(1)]);
        assert_eq!(AeneasTranslator::eval_expr(&tup, &HashMap::new()), 0);
        let app = PureExpr::Apply("f".into(), vec![PureExpr::ConstInt(1)]);
        assert_eq!(AeneasTranslator::eval_expr(&app, &HashMap::new()), 0);
    }

    #[test]
    fn test_permission_lattice_full_outcomes() {
        // None 不可讀;Shared(0.0) 不可讀;Shared(0.3) 可讀不可寫。
        assert!(!Permission::None.is_readable());
        assert!(!Permission::Shared(0.0).is_readable());
        assert!(Permission::Shared(0.3).is_readable());
        assert!(!Permission::Shared(0.3).is_writable());
        // split:Shared(q) 對半;None 拒絕。
        let (a, b) = Permission::Shared(0.25).split().unwrap();
        assert_eq!(a, Permission::Shared(0.125));
        assert_eq!(b, Permission::Shared(0.125));
        assert!(Permission::None.split().is_err());
        assert!(Permission::Shared(0.0).split().is_err());
        // join:None 為單位元;和 <1 為 Shared;和 >1 違反守恆律;Exclusive 衝突。
        let none = Permission::None;
        assert_eq!(
            none.join(&Permission::Shared(0.4)),
            Ok(Permission::Shared(0.4))
        );
        assert_eq!(
            Permission::Shared(0.4).join(&none),
            Ok(Permission::Shared(0.4))
        );
        assert_eq!(
            Permission::Shared(0.4).join(&Permission::Shared(0.25)),
            Ok(Permission::Shared(0.65))
        );
        assert_eq!(
            Permission::Shared(0.4).join(&Permission::Shared(0.7)),
            Err("權限總和 1.1 超出 1.0 (違反分離邏輯守恆律)".to_string())
        );
        assert_eq!(
            Permission::Exclusive.join(&Permission::Shared(0.2)),
            Err("衝突的排他權限合併".to_string())
        );
        // PermissionState:未登記 place 恆為 None;登記後如實取回。
        let mut st = PermissionState::new();
        assert_eq!(st.get_permission("x"), Permission::None);
        st.set_permission("x", Permission::Shared(0.5));
        assert_eq!(st.get_permission("x"), Permission::Shared(0.5));
        // resolve_borrow:未登記變數回 None。
        let mut penv = crate::proof_resources::ProphecyEnvironment::new();
        assert_eq!(penv.resolve_borrow("ghost"), None);
    }
}
