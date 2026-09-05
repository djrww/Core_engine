//! # 有向無環圖項 (DAG Term Representation & Hash-Consing Pool)
//!
//! 將一階項表示為有向無環圖 (DAG)。當兩個或多個分支共享同一個子項時，
//! 透過全局 Hash-Consing 機制僅作指針複製 (Pointer Sharing)，實現：
//!   1. 子項等價性 $O(1)$ 指針/ID 判定 (Pointer Equality)
//!   2. 內存極致緊湊 (Zero Redundant Subterm Allocations)
//!   3. 支持大規模重寫系統中的深層項快速遍歷

use std::collections::HashMap;
use std::fmt;

/// DAG 項的全局唯一指針標識符 (Hash-Consed Unique Term ID)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TermId(pub u32);

/// DAG 節點種類 (不可變結構元)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DagNode {
    /// 變量 (Variable x0, x1, ...)
    Var(u32),
    /// 常量符號 (Constant / 0-ary function)
    Const(String),
    /// 複合函數符號 (Function application: f(t1, ..., tn))
    App(String, Vec<TermId>),
}

/// DAG 項共享池 (Hash-Consing Term Pool)
#[derive(Clone, Debug, Default)]
pub struct DagPool {
    /// 節點存儲數組 (按 TermId 索引)
    nodes: Vec<DagNode>,
    /// 反向哈希表: 節點內容 -> TermId (保證唯一性與指針共享)
    node_to_id: HashMap<DagNode, TermId>,
}

impl DagPool {
    /// 創建一個新的空的 DAG 項共享池
    pub fn new() -> Self {
        DagPool {
            nodes: Vec::new(),
            node_to_id: HashMap::new(),
        }
    }

    /// 獲取項池中的總節點數
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// 判斷項池是否為空
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// 插入或復用變量項 (Variable)
    pub fn var(&mut self, var_idx: u32) -> TermId {
        self.intern(DagNode::Var(var_idx))
    }

    /// 插入或復用常量項 (Constant)
    pub fn constant(&mut self, name: &str) -> TermId {
        self.intern(DagNode::Const(name.to_string()))
    }

    /// 插入或復用函數應用項 (Function Application)
    pub fn app(&mut self, symbol: &str, args: Vec<TermId>) -> TermId {
        if args.is_empty() {
            self.constant(symbol)
        } else {
            self.intern(DagNode::App(symbol.to_string(), args))
        }
    }

    /// 核心 Hash-Consing 實習化例程: 存在即復用指針，否則分配新 ID
    pub fn intern(&mut self, node: DagNode) -> TermId {
        if let Some(&existing_id) = self.node_to_id.get(&node) {
            existing_id
        } else {
            let id = TermId(self.nodes.len() as u32);
            self.nodes.push(node.clone());
            self.node_to_id.insert(node, id);
            id
        }
    }

    /// 獲取節點的底層引用
    pub fn get(&self, id: TermId) -> &DagNode {
        &self.nodes[id.0 as usize]
    }

    /// 判斷兩個項在指針層面上是否完全等價 ($O(1)$ 判定)
    pub fn ptr_eq(&self, t1: TermId, t2: TermId) -> bool {
        t1 == t2
    }

    /// 獲取項的前序遍歷符號序列 (用於辨別樹索引 Linearized Preorder Path)
    pub fn preorder(&self, id: TermId) -> Vec<PreorderSymbol> {
        let mut symbols = Vec::new();
        self.collect_preorder(id, &mut symbols);
        symbols
    }

    fn collect_preorder(&self, id: TermId, out: &mut Vec<PreorderSymbol>) {
        match self.get(id) {
            DagNode::Var(v) => out.push(PreorderSymbol::Var(*v)),
            DagNode::Const(c) => out.push(PreorderSymbol::Sym(c.clone(), 0)),
            DagNode::App(f, args) => {
                out.push(PreorderSymbol::Sym(f.clone(), args.len()));
                for &arg in args {
                    self.collect_preorder(arg, out);
                }
            }
        }
    }

    /// 計算項的結構大小 (包含共享節點在內的完整樹大小)
    pub fn tree_size(&self, id: TermId) -> usize {
        match self.get(id) {
            DagNode::Var(_) | DagNode::Const(_) => 1,
            DagNode::App(_, args) => 1 + args.iter().map(|&a| self.tree_size(a)).sum::<usize>(),
        }
    }

    /// 收集項中的所有自由變量
    pub fn vars_of(&self, id: TermId) -> Vec<u32> {
        let mut vars = Vec::new();
        self.collect_vars(id, &mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    fn collect_vars(&self, id: TermId, out: &mut Vec<u32>) {
        match self.get(id) {
            DagNode::Var(v) => out.push(*v),
            DagNode::Const(_) => {}
            DagNode::App(_, args) => {
                for &arg in args {
                    self.collect_vars(arg, out);
                }
            }
        }
    }

    /// 將 DAG 項格式化為數學字符串表示
    pub fn format_term(&self, id: TermId) -> String {
        match self.get(id) {
            DagNode::Var(v) => format!("x{}", v),
            DagNode::Const(c) => c.clone(),
            DagNode::App(f, args) => {
                let formatted_args: Vec<String> =
                    args.iter().map(|&a| self.format_term(a)).collect();
                format!("{}({})", f, formatted_args.join(", "))
            }
        }
    }
}

/// 前序符號元（供辨別樹索引與快速模式匹配使用）
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PreorderSymbol {
    Sym(String, usize), // (符號名, 元數 arity)
    Var(u32),           // 具名變量
    Wildcard,           // 萬用通配符
}

impl fmt::Display for PreorderSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreorderSymbol::Sym(s, arity) => write!(f, "{}/{}", s, arity),
            PreorderSymbol::Var(v) => write!(f, "x{}", v),
            PreorderSymbol::Wildcard => write!(f, "*"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_pointer_sharing_and_hash_consing() {
        let mut pool = DagPool::new();

        // 構造子項 g(a)
        let a = pool.constant("a");
        let ga1 = pool.app("g", vec![a]);
        let ga2 = pool.app("g", vec![a]);

        // 驗證 Hash-Consing: ga1 與 ga2 獲得完全相同的 TermId (指針共享)
        assert_eq!(ga1, ga2);
        assert!(pool.ptr_eq(ga1, ga2));

        // 構造 f(g(a), g(a))
        let f_term = pool.app("f", vec![ga1, ga2]);

        // 總池中節點數僅為 3: a, g(a), f(g(a), g(a))
        assert_eq!(pool.len(), 3);
        assert_eq!(pool.format_term(f_term), "f(g(a), g(a))");
        assert_eq!(pool.tree_size(f_term), 5); // 根(1) + 左(2) + 右(2) = 5
    }

    #[test]
    fn test_preorder_linearization() {
        let mut pool = DagPool::new();
        let x0 = pool.var(0);
        let b = pool.constant("b");
        let t = pool.app("f", vec![x0, b]);

        let pre = pool.preorder(t);
        assert_eq!(
            pre,
            vec![
                PreorderSymbol::Sym("f".to_string(), 2),
                PreorderSymbol::Var(0),
                PreorderSymbol::Sym("b".to_string(), 0),
            ]
        );
    }
}
