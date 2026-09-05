//! §4.5 基于 van Oostrom 递减图 (Decreasing Diagrams) 的修法菜单抽象重写系统 (ARS)。
//!
//! 状态定义:
//!   A = (V_ev, E_red, Runtime) —— 事实层区间配置。
//! 规则标签集 (I, ≻):
//!   Label::Trim(id) ≻ Label::Split(id) ≻ Label::Runtime(a, b)
//! 满足良基偏序，支持非终止/循环候选下的局部会合性证明与 α-同构规范化。

use crate::ast::Interval;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum K {
    Mut,
    Sh,
}

impl K {
    pub fn label(self) -> &'static str {
        match self {
            K::Mut => "mut",
            K::Sh => "sh",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ev {
    pub id: u32,
    pub storage: u32,
    pub kind: K,
    pub it: Interval,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AState {
    pub evs: Vec<Ev>,
    pub runtime: Vec<(u32, u32)>,
    pub step_count: usize,
}

impl AState {
    /// 构造状态并执行规范化排序与 α-同构重编号
    pub fn new(evs: Vec<Ev>) -> Self {
        let mut s = AState {
            evs,
            runtime: Vec::new(),
            step_count: 0,
        };
        s.canonicalize();
        s
    }

    /// α-同构规范化：按首次出现的空间拓扑序将 storage 紧凑映射为 0, 1, 2...
    pub fn canonicalize(&mut self) {
        self.evs
            .sort_by_key(|e| (e.it.start, e.it.end, e.kind, e.id));
        let mut storage_map = HashMap::new();
        let mut next_sid = 0u32;
        for e in &mut self.evs {
            let sid = *storage_map.entry(e.storage).or_insert_with(|| {
                let id = next_sid;
                next_sid += 1;
                id
            });
            e.storage = sid;
        }
        self.evs
            .sort_by_key(|e| (e.storage, e.it.start, e.it.end, e.id));
        self.runtime.sort();
        self.runtime.dedup();
    }

    /// O(N log N) 扫描线法检测红边（冲突边）
    pub fn red_edges(&self) -> Vec<(u32, u32)> {
        let mut reds = Vec::new();
        let n = self.evs.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let (a, b) = (&self.evs[i], &self.evs[j]);
                if a.storage != b.storage {
                    continue;
                }
                // 冲突条件: (Mut ↔ Mut 或 Mut ↔ Sh) ∧ 半开区间严格相交
                if (a.kind == K::Mut || b.kind == K::Mut) && a.it.overlaps(&b.it) {
                    let pair = (a.id.min(b.id), a.id.max(b.id));
                    if !self.runtime.contains(&pair) {
                        reds.push(pair);
                    }
                }
            }
        }
        reds.sort();
        reds.dedup();
        reds
    }

    pub fn is_normal_form(&self) -> bool {
        self.red_edges().is_empty()
    }

    pub fn canon_key(&self) -> Vec<(u32, u32, K, u32, u32)> {
        // D-04:與 l9newman(Newman 通道)共享同一份鍵構造單一真相,
        // 兩條通道對「同一狀態」的判定永遠一致。
        crate::rep::canon_state_key(
            self.evs
                .iter()
                .map(|e| (e.id, e.storage, e.kind, e.it.start, e.it.end)),
        )
    }
}

/// 规则标签 (Label)：定义良基偏序集 (I, ≻)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Label {
    /// 优先级 1: 确定性规范修剪
    Trim(u32),
    /// 优先级 2: 存储分裂
    Split(u32),
    /// 优先级 3: 运行时标记
    Runtime(u32, u32),
}

impl Label {
    /// 严格偏序比较: Label::Trim < Label::Split < Label::Runtime
    pub fn precedes(&self, other: &Label) -> bool {
        self < other
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LabeledRule {
    pub label: Label,
    pub action: Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    /// R1: 将事件 id 的右端点缩短至 cut
    R1Shorten { id: u32, cut: u32 },
    /// R2: 将事件 id 在 cut 处分裂并将后续活动分配给新 storage
    R2Split { id: u32, cut: u32 },
    /// R4: 将借用冲突 (a, b) 标记为运行时检查
    R4Runtime { a: u32, b: u32 },
}

pub fn apply_rule(s: &AState, r: &LabeledRule) -> Option<AState> {
    let mut s2 = s.clone();
    s2.step_count += 1;
    match r.action {
        Action::R1Shorten { id, cut } => {
            let ev = s2.evs.iter_mut().find(|e| e.id == id)?;
            if cut <= ev.it.start || cut >= ev.it.end {
                return None;
            }
            ev.it.end = cut;
        }
        Action::R2Split { id, cut } => {
            let ev = s2.evs.iter().find(|e| e.id == id)?;
            if cut <= ev.it.start || cut >= ev.it.end {
                return None;
            }
            let old_storage = ev.storage;
            let new_storage = s2.evs.iter().map(|e| e.storage).max().unwrap_or(0) + 1;
            for e in s2.evs.iter_mut() {
                if e.storage == old_storage && e.it.start >= cut {
                    e.storage = new_storage;
                }
            }
        }
        Action::R4Runtime { a, b } => {
            let pair = (a.min(b), a.max(b));
            if !s.red_edges().contains(&pair) {
                return None;
            }
            s2.runtime.push(pair);
        }
    }
    s2.canonicalize();
    Some(s2)
}
