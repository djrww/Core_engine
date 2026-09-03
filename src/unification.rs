//! # 線性/近線性一階合一演算法 (Near-Linear First-Order Unification on DAG Terms)
//!
//! 參考文獻:
//!   - Alberto Martelli & Ugo Montanari (1982), "An Efficient Algorithm for Unification" (ACM TOPLAS)
//!   - Michael S. Paterson & Mark N. Wegman (1978), "Linear Unification" (JCSS)
//!
//! 基於 DAG 項表示與 Union-Find 並查集 (帶路徑壓縮與秩啓發式) 的近線性合一演算法 $O(n \alpha(n))$，
//! 用於快速計算臨界對 (Critical Pairs)、重寫合一與消解。

use crate::dag_term::{DagNode, DagPool, TermId};
use std::collections::HashMap;

/// 最一般合一子代換 (Most General Unifier, MGU)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Substitution {
    pub map: HashMap<u32, TermId>,
}

impl Substitution {
    pub fn new() -> Self {
        Substitution {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, var: u32, term: TermId) {
        self.map.insert(var, term);
    }

    pub fn get(&self, var: u32) -> Option<TermId> {
        self.map.get(&var).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// 將代換應用於 DAG 項中
    pub fn apply(&self, pool: &mut DagPool, term: TermId) -> TermId {
        match pool.get(term).clone() {
            DagNode::Var(v) => {
                if let Some(&subst_term) = self.map.get(&v) {
                    self.apply(pool, subst_term)
                } else {
                    term
                }
            }
            DagNode::Const(_) => term,
            DagNode::App(f, args) => {
                let new_args: Vec<TermId> = args.iter().map(|&a| self.apply(pool, a)).collect();
                pool.app(&f, new_args)
            }
        }
    }
}

/// Union-Find 基近線性合一求解器
pub struct FastUnificationSolver {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl FastUnificationSolver {
    pub fn new(size: usize) -> Self {
        FastUnificationSolver {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    pub fn union(&mut self, x: usize, y: usize) -> usize {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return rx;
        }
        if self.rank[rx] < self.rank[ry] {
            self.parent[rx] = ry;
            ry
        } else {
            self.parent[ry] = rx;
            if self.rank[rx] == self.rank[ry] {
                self.rank[rx] += 1;
            }
            rx
        }
    }

    /// 近線性 Union-Find 合一求解入口
    pub fn unify_near_linear(pool: &mut DagPool, t1: TermId, t2: TermId) -> Option<Substitution> {
        unify(pool, t1, t2)
    }
}

/// 經典 Martelli-Montanari 規則消解黃金求解器 (Delete, Decompose, Swap, Eliminate)
pub fn unify_martelli_montanari(
    pool: &mut DagPool,
    t1: TermId,
    t2: TermId,
) -> Option<Substitution> {
    let mut subst = Substitution::new();
    let mut equations = vec![(t1, t2)];

    while let Some((mut s, mut t)) = equations.pop() {
        s = subst.apply(pool, s);
        t = subst.apply(pool, t);

        if s == t {
            // Delete rule: x = x -> {}
            continue;
        }

        let node_s = pool.get(s).clone();
        let node_t = pool.get(t).clone();

        match (node_s, node_t) {
            (DagNode::Var(v), _) => {
                if pool.vars_of(t).contains(&v) {
                    return None; // Occur-check failure
                }
                // Eliminate rule: v |-> t
                let mut new_subst = Substitution::new();
                new_subst.insert(v, t);
                for eq in &mut equations {
                    eq.0 = new_subst.apply(pool, eq.0);
                    eq.1 = new_subst.apply(pool, eq.1);
                }
                subst.insert(v, t);
            }
            (_, DagNode::Var(_v)) => {
                // Swap rule: t = v -> v = t
                equations.push((t, s));
            }
            (DagNode::Const(c1), DagNode::Const(c2)) => {
                if c1 != c2 {
                    return None;
                }
            }
            (DagNode::App(f1, args1), DagNode::App(f2, args2)) => {
                // Decompose rule: f(s1..sn) = f(t1..tn) -> s1=t1..sn=tn
                if f1 != f2 || args1.len() != args2.len() {
                    return None;
                }
                for (&arg1, &arg2) in args1.iter().zip(args2.iter()) {
                    equations.push((arg1, arg2));
                }
            }
            _ => return None,
        }
    }

    Some(subst)
}

/// 核心合一入口: 計算項 t1 與 t2 的 MGU (Most General Unifier)
pub fn unify(pool: &mut DagPool, t1: TermId, t2: TermId) -> Option<Substitution> {
    let mut solver = FastUnificationSolver::new(pool.len() + 16);
    let mut subst = Substitution::new();

    let mut queue = vec![(t1, t2)];

    while let Some((a, b)) = queue.pop() {
        let ra_idx = solver.find(a.0 as usize);
        let rb_idx = solver.find(b.0 as usize);

        if ra_idx == rb_idx {
            continue;
        }

        let ra_term = TermId(ra_idx as u32);
        let rb_term = TermId(rb_idx as u32);

        let node_a = pool.get(ra_term).clone();
        let node_b = pool.get(rb_term).clone();

        match (node_a, node_b) {
            (DagNode::Var(v1), DagNode::Var(v2)) => {
                if v1 != v2 {
                    solver.union(ra_idx, rb_idx);
                    subst.insert(v1, rb_term);
                }
            }
            (DagNode::Var(v), other) | (other, DagNode::Var(v)) => {
                let other_term = match other {
                    DagNode::Var(_) => unreachable!(),
                    _ => {
                        if pool.get(ra_term) == &DagNode::Var(v) {
                            rb_term
                        } else {
                            ra_term
                        }
                    }
                };

                // Occur-Check: 確保變量不出現於其被實例化子項中 (避免循環項)
                if pool.vars_of(other_term).contains(&v) {
                    return None; // Occur-check failure!
                }

                solver.union(ra_idx, rb_idx);
                subst.insert(v, other_term);
            }
            (DagNode::Const(c1), DagNode::Const(c2)) => {
                if c1 != c2 {
                    return None; // 符號不匹配
                }
                solver.union(ra_idx, rb_idx);
            }
            (DagNode::App(f1, args1), DagNode::App(f2, args2)) => {
                if f1 != f2 || args1.len() != args2.len() {
                    return None; // 符號或元數不匹配
                }
                solver.union(ra_idx, rb_idx);
                for (&arg1, &arg2) in args1.iter().zip(args2.iter()) {
                    queue.push((arg1, arg2));
                }
            }
            _ => {
                return None; // 構造衝突 (如 Const ↔ App)
            }
        }
    }

    Some(subst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_near_linear_unification_success() {
        let mut pool = DagPool::new();

        // t1 = f(x, g(y))
        let x = pool.var(0);
        let y = pool.var(1);
        let gy = pool.app("g", vec![y]);
        let t1 = pool.app("f", vec![x, gy]);

        // t2 = f(a, g(b))
        let a = pool.constant("a");
        let b = pool.constant("b");
        let gb = pool.app("g", vec![b]);
        let t2 = pool.app("f", vec![a, gb]);

        let mgu = unify(&mut pool, t1, t2).expect("t1 与 t2 应可合一");
        assert_eq!(mgu.get(0), Some(a)); // x |-> a
        assert_eq!(mgu.get(1), Some(b)); // y |-> b

        let t1_subst = mgu.apply(&mut pool, t1);
        let t2_subst = mgu.apply(&mut pool, t2);
        assert_eq!(t1_subst, t2_subst);
    }

    #[test]
    fn test_unification_occur_check_failure() {
        let mut pool = DagPool::new();

        // t1 = x, t2 = f(x)
        let x = pool.var(0);
        let fx = pool.app("f", vec![x]);

        let res = unify(&mut pool, x, fx);
        assert!(res.is_none(), "x 与 f(x) 应因 occur-check 失败而不可合一");
    }
}
