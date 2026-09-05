//! # 辨別樹與指針倒排索引 (Discrimination Trees & Pointer Inverted Index)
//!
//! 參考文獻:
//!   - Peter Graf (1995), "Term Indexing" (LNAI 1053)
//!   - William McCune (1992), "Experiments with Discrimination-Tree Indexing and Path Indexing for Term Retrieval"
//!
//! 提供定理證明與重寫系統中的核心項索引結構 (Term Indexing)：
//!   1. **辨別樹 (Discrimination Tree / Trie)**: 將項前序序列前綴共享，在 $O(\text{depth})$ 內檢索可應用的重寫規則
//!   2. **泛化查詢 (Generalizations Retrieval)**: 檢索左側模式匹配當前目標項的規則 (Pattern Matching for Rewrite Steps)
//!   3. **合一查詢 (Unifiables Retrieval)**: 快速過濾可合一項集 (Critical Pair Extraction)
//!   4. **指針倒排索引 (Pointer Inverted Index)**: 按 TermId 建立直接引用倒排索引，加速項替換與重寫

use crate::dag_term::{DagPool, PreorderSymbol, TermId};
use std::collections::{BTreeMap, HashMap};

/// 辨別樹節點
#[derive(Clone, Debug)]
pub struct DtNode<V> {
    /// 當前節點直接掛載的值 (通常為重寫規則或等式)
    pub values: Vec<V>,
    /// 子分支: 符號 -> 子樹
    pub children: BTreeMap<PreorderSymbol, DtNode<V>>,
}

impl<V> Default for DtNode<V> {
    fn default() -> Self {
        DtNode {
            values: Vec::new(),
            children: BTreeMap::new(),
        }
    }
}

/// 辨別樹索引結構 (Discrimination Tree Index)
#[derive(Clone, Debug)]
pub struct DiscriminationTree<V> {
    pub root: DtNode<V>,
    pub size: usize,
}

impl<V> Default for DiscriminationTree<V> {
    fn default() -> Self {
        DiscriminationTree {
            root: DtNode::default(),
            size: 0,
        }
    }
}

impl<V: Clone> DiscriminationTree<V> {
    /// 創建一個空的辨別樹索引
    pub fn new() -> Self {
        Self::default()
    }

    /// 將 DAG 項及其關聯值 (如重寫規則) 插入索引中
    pub fn insert(&mut self, pool: &DagPool, term: TermId, value: V) {
        let path = pool.preorder(term);
        // 將具體變量標準化為通配符 Wildcard，以支持模式泛化索引
        let normalized_path: Vec<PreorderSymbol> = path
            .into_iter()
            .map(|sym| match sym {
                PreorderSymbol::Var(_) => PreorderSymbol::Wildcard,
                s => s,
            })
            .collect();

        let mut curr = &mut self.root;
        for sym in normalized_path {
            curr = curr.children.entry(sym).or_default();
        }
        curr.values.push(value);
        self.size += 1;
    }

    /// 泛化查詢 (Generalizations Retrieval):
    /// 檢索所有索引中可以匹配 (Match) 目標項 `target` 的模式項對應的值。
    /// 即檢索滿足 $t_{\text{indexed}}\sigma = t_{\text{target}}$ 的所有條目 (重寫規則匹配核心)。
    pub fn query_generalizations(&self, pool: &DagPool, target: TermId) -> Vec<V> {
        let target_path = pool.preorder(target);
        let mut results = Vec::new();
        Self::match_generalizations(&self.root, &target_path, 0, &mut results);
        results
    }

    fn match_generalizations(
        node: &DtNode<V>,
        target_path: &[PreorderSymbol],
        cursor: usize,
        out: &mut Vec<V>,
    ) {
        if cursor >= target_path.len() {
            out.extend(node.values.iter().cloned());
            return;
        }

        let curr_sym = &target_path[cursor];

        // 1. 精確符號匹配
        if let Some(child) = node.children.get(curr_sym) {
            Self::match_generalizations(child, target_path, cursor + 1, out);
        }

        // 2. 通配符/變量匹配 (Wildcard 能吞噬整個子項跨度)
        if let Some(wildcard_child) = node.children.get(&PreorderSymbol::Wildcard) {
            let skip_len = Self::subterm_preorder_length(target_path, cursor);
            Self::match_generalizations(wildcard_child, target_path, cursor + skip_len, out);
        }
    }

    /// 計算給定前序符號序列中，從 cursor 開始的完整子項所佔的符號總長度
    fn subterm_preorder_length(path: &[PreorderSymbol], cursor: usize) -> usize {
        if cursor >= path.len() {
            return 0;
        }
        match &path[cursor] {
            PreorderSymbol::Var(_) | PreorderSymbol::Wildcard => 1,
            PreorderSymbol::Sym(_, arity) => {
                let mut len = 1;
                let mut remaining_args = *arity;
                while remaining_args > 0 && cursor + len < path.len() {
                    let arg_len = Self::subterm_preorder_length(path, cursor + len);
                    len += arg_len;
                    remaining_args -= 1;
                }
                len
            }
        }
    }
}

/// 指針倒排索引 (Pointer Inverted Index)
///
/// 記錄每個子項 TermId 被哪些父項或規則引用，支持 $O(1)$ 影響傳播與快速項改寫。
#[derive(Clone, Debug, Default)]
pub struct PointerInvertedIndex {
    /// TermId -> 引用該項的父項集合
    pub parents: HashMap<TermId, Vec<TermId>>,
    /// TermId -> 關聯的標籤/規則 ID 列表
    pub term_to_rules: HashMap<TermId, Vec<u32>>,
}

impl PointerInvertedIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// 索引項池中的父子關聯
    pub fn index_pool(&mut self, pool: &DagPool) {
        self.parents.clear();
        for id_idx in 0..pool.len() {
            let id = TermId(id_idx as u32);
            if let crate::dag_term::DagNode::App(_, ref args) = pool.get(id) {
                for &arg in args {
                    self.parents.entry(arg).or_default().push(id);
                }
            }
        }
    }

    /// 關聯項與規則 ID
    pub fn register_rule_term(&mut self, term: TermId, rule_id: u32) {
        self.term_to_rules.entry(term).or_default().push(rule_id);
    }

    /// 獲取所有直接引用 `term` 的父項
    pub fn get_parents(&self, term: TermId) -> &[TermId] {
        self.parents.get(&term).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discrimination_tree_indexing_and_generalization() {
        let mut pool = DagPool::new();

        // 構造索引項/規則 1: f(x, a) -> Rule 1
        let x = pool.var(0);
        let a = pool.constant("a");
        let pat1 = pool.app("f", vec![x, a]);

        // 構造索引項/規則 2: f(b, y) -> Rule 2
        let b = pool.constant("b");
        let y = pool.var(1);
        let pat2 = pool.app("f", vec![b, y]);

        let mut dt: DiscriminationTree<&'static str> = DiscriminationTree::new();
        dt.insert(&pool, pat1, "Rule-1: f(x, a)");
        dt.insert(&pool, pat2, "Rule-2: f(b, y)");

        // 構造目標項 target = f(b, a)
        // 應同時匹配 Rule 1 (x=b, a=a) 與 Rule 2 (b=b, y=a)
        let target = pool.app("f", vec![b, a]);
        let matches = dt.query_generalizations(&pool, target);

        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&"Rule-1: f(x, a)"));
        assert!(matches.contains(&"Rule-2: f(b, y)"));

        // 構造目標項 target2 = f(c, a)
        let c = pool.constant("c");
        let target2 = pool.app("f", vec![c, a]);
        let matches2 = dt.query_generalizations(&pool, target2);

        assert_eq!(matches2.len(), 1);
        assert_eq!(matches2[0], "Rule-1: f(x, a)");
    }

    #[test]
    fn test_pointer_inverted_index() {
        let mut pool = DagPool::new();
        let a = pool.constant("a");
        let ga = pool.app("g", vec![a]);
        let fga = pool.app("f", vec![ga, a]);

        let mut pii = PointerInvertedIndex::new();
        pii.index_pool(&pool);

        // a 的父節點應為 g(a) 與 f(g(a), a)
        let parents_of_a = pii.get_parents(a);
        assert!(parents_of_a.contains(&ga));
        assert!(parents_of_a.contains(&fga));
    }
}
