//! §8.1 Mid-level Intermediate Representation (MIR) 核心控制流圖、Move 分析、Drop 展開與 NLL 借用檢查器。
//!
//! 對齊 Rustc 編譯器架構核心原則：
//!   1. Borrow checker、NLL、drop elaboration、move analysis 全都在 MIR (而非語法 AST) 上執行。
//!   2. MIR 基於控制流圖 (CFG)，以基本塊 (BasicBlock)、陳述句 (Statement) 與終結符 (Terminator) 構成。
//!   3. Move Path 數據流分析：基於 Place (地方) 追蹤 DefinitelyInit / MaybeUninit / Moved 狀態。
//!   4. Drop Elaboration：嚴格落實宣告反序 (Reverse Declaration) 與結構體字段正序 (Field Declaration) 的 Dropck 順序，
//!      並在條件初始化分支中生成動態 Drop Flag。
//!   5. NLL (Non-Lexical Lifetimes) 區域推導：基於 CFG 點生成 outlives 子類型約束並核驗 Loan 失效點。

use crate::span::Span;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

/// MIR 局部變量標識 (_0 為返回值, _1.._n 為參數與局部變量)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local(pub u32);

impl Display for Local {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "_{}", self.0)
    }
}

/// 基本塊標識 (bb0, bb1, ...)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BasicBlock(pub u32);

impl Display for BasicBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// MIR 區域/生命週期變量
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionVid(pub u32);

impl Display for RegionVid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "'r{}", self.0)
    }
}

/// MIR 類型系統
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MirType {
    Bool,
    Int(usize), // 位寬，如 32 表示 i32
    Uint(usize),
    TypeParam(String),     // 泛型型別參數 T
    LifetimeParam(String), // 泛型生命週期參數 'a
    Tuple(Vec<MirType>),
    Adt {
        name: String,
        fields: Vec<(String, MirType)>,
    },
    Ref(RegionVid, Box<MirType>, BorrowKind),
    RawPtr(Box<MirType>, bool), // (type, is_mut)
    Array(Box<MirType>, usize),
    Slice(Box<MirType>),
    FnPtr {
        params: Vec<MirType>,
        ret: Box<MirType>,
    },
    Never,
}

/// 投影元：從基底 Local 導出深層 Place 的路徑
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProjectionElem {
    Deref,
    Field(u32),
    Index(Local),
    Downcast(u32),
}

/// Place (地方)：表示記憶體中具備位置語義的儲存單元
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Place {
    pub local: Local,
    pub projection: Vec<ProjectionElem>,
}

impl Place {
    pub fn from_local(local: Local) -> Self {
        Self {
            local,
            projection: Vec::new(),
        }
    }

    pub fn field(mut self, field_idx: u32) -> Self {
        self.projection.push(ProjectionElem::Field(field_idx));
        self
    }

    pub fn deref(mut self) -> Self {
        self.projection.push(ProjectionElem::Deref);
        self
    }

    /// 判斷 self 是否為 other 的前綴 (Prefix)
    pub fn is_prefix_of(&self, other: &Place) -> bool {
        if self.local != other.local {
            return false;
        }
        if self.projection.len() > other.projection.len() {
            return false;
        }
        self.projection
            .iter()
            .zip(other.projection.iter())
            .all(|(p1, p2)| p1 == p2)
    }

    /// 判斷兩個 Place 是否可能重疊 (Overlap / Conflicting Access)
    pub fn overlaps(&self, other: &Place) -> bool {
        self.is_prefix_of(other) || other.is_prefix_of(self)
    }
}

impl Display for Place {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = format!("{}", self.local);
        for proj in &self.projection {
            match proj {
                ProjectionElem::Deref => s = format!("(*{})", s),
                ProjectionElem::Field(idx) => s = format!("({}.{})", s, idx),
                ProjectionElem::Index(loc) => s = format!("{}[{}]", s, loc),
                ProjectionElem::Downcast(var) => s = format!("({} as variant#{})", s, var),
            }
        }
        write!(f, "{}", s)
    }
}

/// 借用種類
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BorrowKind {
    Shared,
    Mut { allow_two_phase_borrow: bool },
    Shallow,
    Unique,
}

/// 二元運算符
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MirBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
    Offset,
}

/// 一元運算符
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MirUnOp {
    Not,
    Neg,
}

/// 常量值
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConstantKind {
    Bool(bool),
    Int(i128),
    Uint(u128),
    Str(String),
}

/// 操作數 (Operand)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(ConstantKind),
}

/// 右值 (Rvalue)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Rvalue {
    Use(Operand),
    Ref(RegionVid, BorrowKind, Place),
    BinaryOp(MirBinOp, Operand, Operand),
    UnaryOp(MirUnOp, Operand),
    Discriminant(Place),
    Aggregate(Vec<Operand>),
    Len(Place),
}

/// 陳述句 (Statement)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatementKind {
    Assign(Place, Rvalue),
    StorageLive(Local),
    StorageDead(Local),
    Deinit(Place),
    SetDiscriminant { place: Place, variant_index: u32 },
    Retag(BorrowKind, Place),
    Nop,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

/// 分支目標
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwitchTargets {
    pub values: Vec<u128>,
    pub targets: Vec<BasicBlock>,
    pub otherwise: BasicBlock,
}

/// 終結符 (Terminator)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminatorKind {
    Goto {
        target: BasicBlock,
    },
    SwitchInt {
        discr: Operand,
        targets: SwitchTargets,
    },
    Return,
    Unreachable,
    Drop {
        place: Place,
        target: BasicBlock,
        unwind: Option<BasicBlock>,
    },
    Call {
        func: Operand,
        args: Vec<Operand>,
        destination: Place,
        target: Option<BasicBlock>,
        cleanup: Option<BasicBlock>,
    },
    Assert {
        cond: Operand,
        expected: bool,
        msg: String,
        target: BasicBlock,
        cleanup: Option<BasicBlock>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terminator {
    pub kind: TerminatorKind,
    pub span: Span,
}

/// 基本塊數據
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasicBlockData {
    pub statements: Vec<Statement>,
    pub terminator: Option<Terminator>,
    pub is_cleanup: bool,
}

impl BasicBlockData {
    pub fn new(terminator: Option<Terminator>) -> Self {
        Self {
            statements: Vec::new(),
            terminator,
            is_cleanup: false,
        }
    }
}

/// 局部變量宣告元數據
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalDecl {
    pub ty: MirType,
    pub is_mut: bool,
    pub span: Span,
    pub name: Option<String>,
}

/// CFG 位置座標 (Location)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub block: BasicBlock,
    pub statement_index: usize,
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.block, self.statement_index)
    }
}

/// 完整 MIR 函數體 (MIR Body)
#[derive(Clone, Debug)]
pub struct MirBody {
    pub basic_blocks: Vec<BasicBlockData>,
    pub local_decls: Vec<LocalDecl>,
    pub arg_count: usize,
}

impl MirBody {
    pub fn new(arg_count: usize) -> Self {
        Self {
            basic_blocks: Vec::new(),
            local_decls: Vec::new(),
            arg_count,
        }
    }

    pub fn num_blocks(&self) -> usize {
        self.basic_blocks.len()
    }

    pub fn num_locals(&self) -> usize {
        self.local_decls.len()
    }

    pub fn add_local(
        &mut self,
        ty: MirType,
        is_mut: bool,
        span: Span,
        name: Option<String>,
    ) -> Local {
        let idx = self.local_decls.len() as u32;
        self.local_decls.push(LocalDecl {
            ty,
            is_mut,
            span,
            name,
        });
        Local(idx)
    }

    pub fn add_block(&mut self, data: BasicBlockData) -> BasicBlock {
        let idx = self.basic_blocks.len() as u32;
        self.basic_blocks.push(data);
        BasicBlock(idx)
    }

    /// 提取 CFG 所有前驅與後繼關係
    pub fn cfg_successors(&self, bb: BasicBlock) -> Vec<BasicBlock> {
        let mut succs = Vec::new();
        if let Some(block) = self.basic_blocks.get(bb.0 as usize) {
            if let Some(ref term) = block.terminator {
                match &term.kind {
                    TerminatorKind::Goto { target } => succs.push(*target),
                    TerminatorKind::SwitchInt { targets, .. } => {
                        succs.extend(&targets.targets);
                        succs.push(targets.otherwise);
                    }
                    TerminatorKind::Drop { target, unwind, .. } => {
                        succs.push(*target);
                        if let Some(unw) = unwind {
                            succs.push(*unw);
                        }
                    }
                    TerminatorKind::Call {
                        target, cleanup, ..
                    } => {
                        if let Some(t) = target {
                            succs.push(*t);
                        }
                        if let Some(c) = cleanup {
                            succs.push(*c);
                        }
                    }
                    TerminatorKind::Assert {
                        target, cleanup, ..
                    } => {
                        succs.push(*target);
                        if let Some(c) = cleanup {
                            succs.push(*c);
                        }
                    }
                    TerminatorKind::Return | TerminatorKind::Unreachable => {}
                }
            }
        }
        succs
    }
}

// =========================================================================
// Move Path 數據流分析 (DefinitelyInit / MaybeUninit / Move Path Hierarchy)
// =========================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MovePathIndex(pub u32);

#[derive(Clone, Debug)]
pub struct MovePath {
    pub place: Place,
    pub parent: Option<MovePathIndex>,
    pub first_child: Option<MovePathIndex>,
    pub next_sibling: Option<MovePathIndex>,
}

#[derive(Clone, Debug, Default)]
pub struct MoveData {
    pub move_paths: Vec<MovePath>,
    pub path_map: HashMap<Place, MovePathIndex>,
}

impl MoveData {
    pub fn build(body: &MirBody) -> Self {
        let mut md = MoveData::default();
        for i in 0..body.num_locals() {
            let place = Place::from_local(Local(i as u32));
            md.get_or_insert_path(place);
        }
        md
    }

    pub fn get_or_insert_path(&mut self, place: Place) -> MovePathIndex {
        if let Some(&idx) = self.path_map.get(&place) {
            return idx;
        }

        let parent_idx = if place.projection.is_empty() {
            None
        } else {
            let mut parent_place = place.clone();
            parent_place.projection.pop();
            Some(self.get_or_insert_path(parent_place))
        };

        let new_idx = MovePathIndex(self.move_paths.len() as u32);
        self.move_paths.push(MovePath {
            place: place.clone(),
            parent: parent_idx,
            first_child: None,
            next_sibling: None,
        });
        self.path_map.insert(place, new_idx);
        new_idx
    }
}

/// 初始化狀態標記
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitStatus {
    DefinitelyInitialized,
    MaybeUninitialized,
    MovedOut(Location),
}

/// Move Analysis 求解器
pub struct MoveAnalysisSolver;

impl MoveAnalysisSolver {
    /// 分析特定位置處各 Local 與 Place 的初始化狀態
    pub fn compute_init_states(
        body: &MirBody,
        move_data: &MoveData,
    ) -> HashMap<(Location, MovePathIndex), InitStatus> {
        let mut states: HashMap<(Location, MovePathIndex), InitStatus> = HashMap::new();

        // 參數在入口 bb0[0] 預設為 DefinitelyInitialized
        for arg in 1..=body.arg_count {
            let place = Place::from_local(Local(arg as u32));
            if let Some(&path_idx) = move_data.path_map.get(&place) {
                states.insert(
                    (
                        Location {
                            block: BasicBlock(0),
                            statement_index: 0,
                        },
                        path_idx,
                    ),
                    InitStatus::DefinitelyInitialized,
                );
            }
        }

        for (bb_idx, block) in body.basic_blocks.iter().enumerate() {
            let bb = BasicBlock(bb_idx as u32);
            for (stmt_idx, stmt) in block.statements.iter().enumerate() {
                let loc = Location {
                    block: bb,
                    statement_index: stmt_idx,
                };
                match &stmt.kind {
                    StatementKind::Assign(lhs, rhs) => {
                        if let Some(&path_idx) = move_data.path_map.get(lhs) {
                            states.insert((loc, path_idx), InitStatus::DefinitelyInitialized);
                        }
                        // 如果右值是 Move(p)，則標記 p 及其子路徑為 MovedOut
                        if let Rvalue::Use(Operand::Move(ref moved_place)) = rhs {
                            if let Some(&moved_path) = move_data.path_map.get(moved_place) {
                                states.insert((loc, moved_path), InitStatus::MovedOut(loc));
                            }
                        }
                    }
                    StatementKind::StorageDead(local) => {
                        let place = Place::from_local(*local);
                        if let Some(&path_idx) = move_data.path_map.get(&place) {
                            states.insert((loc, path_idx), InitStatus::MaybeUninitialized);
                        }
                    }
                    _ => {}
                }
            }
        }

        states
    }

    /// 檢驗是否有 Use of Moved Value 或 Use of Uninitialized Memory
    pub fn check_use_validity(
        body: &MirBody,
        move_data: &MoveData,
        init_states: &HashMap<(Location, MovePathIndex), InitStatus>,
    ) -> Vec<(Location, Place, String)> {
        let mut errors = Vec::new();

        for (bb_idx, block) in body.basic_blocks.iter().enumerate() {
            let bb = BasicBlock(bb_idx as u32);
            for (stmt_idx, stmt) in block.statements.iter().enumerate() {
                let loc = Location {
                    block: bb,
                    statement_index: stmt_idx,
                };
                let check_operand = |op: &Operand, errors: &mut Vec<(Location, Place, String)>| {
                    if let Operand::Copy(p) | Operand::Move(p) = op {
                        if let Some(&path_idx) = move_data.path_map.get(p) {
                            if let Some(status) = init_states.get(&(loc, path_idx)) {
                                match status {
                                    InitStatus::MovedOut(move_loc) => {
                                        errors.push((
                                            loc,
                                            p.clone(),
                                            format!(
                                                "E0382: 使用了已被移動的值 `{}` (移動發生於 {})",
                                                p, move_loc
                                            ),
                                        ));
                                    }
                                    InitStatus::MaybeUninitialized => {
                                        errors.push((
                                            loc,
                                            p.clone(),
                                            format!("E0381: 使用了可能未初始化的變量 `{}`", p),
                                        ));
                                    }
                                    InitStatus::DefinitelyInitialized => {}
                                }
                            }
                        }
                    }
                };

                if let StatementKind::Assign(_, rhs) = &stmt.kind {
                    match rhs {
                        Rvalue::Use(op) => check_operand(op, &mut errors),
                        Rvalue::BinaryOp(_, op1, op2) => {
                            check_operand(op1, &mut errors);
                            check_operand(op2, &mut errors);
                        }
                        Rvalue::UnaryOp(_, op) => check_operand(op, &mut errors),
                        Rvalue::Aggregate(ops) => {
                            for op in ops {
                                check_operand(op, &mut errors);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        errors
    }
}

// =========================================================================
// Drop Elaboration 引擎 (宣告反序、字段正序、動態 Drop Flag 展開)
// =========================================================================

#[derive(Clone, Debug)]
pub struct ElaboratedDropSequence {
    pub drops: Vec<(Place, MirType)>,
    pub dynamic_flags_needed: Vec<Local>,
}

pub struct DropElaborator;

impl DropElaborator {
    /// 為給定作用域計算精確的 Drop 順序：
    /// 1. 局部變量按宣告相反順序 (Reverse of declaration)
    /// 2. 結構體字段按定義順序 (Declaration order)
    pub fn elaborate_scope_drops(
        locals_in_scope: &[Local],
        body: &MirBody,
    ) -> ElaboratedDropSequence {
        let mut drops = Vec::new();
        let mut flags = Vec::new();

        // 宣告反序迭代局部變量
        for &local in locals_in_scope.iter().rev() {
            if let Some(decl) = body.local_decls.get(local.0 as usize) {
                let base_place = Place::from_local(local);
                match &decl.ty {
                    MirType::Adt { fields, .. } => {
                        // 字段按宣告正序展開 Drop
                        for (f_idx, (_, f_ty)) in fields.iter().enumerate() {
                            drops.push((base_place.clone().field(f_idx as u32), f_ty.clone()));
                        }
                    }
                    other => {
                        drops.push((base_place, other.clone()));
                    }
                }
                flags.push(local);
            }
        }

        ElaboratedDropSequence {
            drops,
            dynamic_flags_needed: flags,
        }
    }
}

// =========================================================================
// NLL (Non-Lexical Lifetimes) 與 MIR 借用檢查器
// =========================================================================

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoanId(pub u32);

#[derive(Clone, Debug)]
pub struct ActiveLoan {
    pub id: LoanId,
    pub issued_at: Location,
    pub place: Place,
    pub kind: BorrowKind,
    pub region: RegionVid,
}

#[derive(Clone, Debug, Default)]
pub struct MirBorrowckReport {
    pub active_loans: Vec<ActiveLoan>,
    pub outlives_constraints: Vec<(RegionVid, RegionVid, Location)>, // ('r1 : 'r2 @ loc)
    pub borrow_conflicts: Vec<(Location, Place, String)>,
}

pub struct MirBorrowChecker;

impl MirBorrowChecker {
    /// 在 MIR 控制流圖上執行 NLL 借用檢查
    pub fn check_body(body: &MirBody) -> MirBorrowckReport {
        let mut report = MirBorrowckReport::default();
        let mut loan_counter = 0u32;
        let mut live_loans: HashMap<Location, Vec<ActiveLoan>> = HashMap::new();

        for (bb_idx, block) in body.basic_blocks.iter().enumerate() {
            let bb = BasicBlock(bb_idx as u32);
            for (stmt_idx, stmt) in block.statements.iter().enumerate() {
                let loc = Location {
                    block: bb,
                    statement_index: stmt_idx,
                };

                if let StatementKind::Assign(lhs, rhs) = &stmt.kind {
                    // 1. 檢查寫入 lhs 是否破壞現有活躍借用 (Invalidation check)
                    for loans in live_loans.values() {
                        for loan in loans {
                            if loan.place.overlaps(lhs) {
                                report.borrow_conflicts.push((
                                    loc,
                                    lhs.clone(),
                                    format!(
                                        "E0506: 無法賦值給 `{}`，因為它已被借用 (Loan #{:?} 於 {})",
                                        lhs, loan.id, loan.issued_at
                                    ),
                                ));
                            }
                        }
                    }

                    // 2. 如果右值發起借用，記錄 Loan
                    if let Rvalue::Ref(region, kind, ref borrowed_place) = rhs {
                        let loan = ActiveLoan {
                            id: LoanId(loan_counter),
                            issued_at: loc,
                            place: borrowed_place.clone(),
                            kind: *kind,
                            region: *region,
                        };
                        loan_counter += 1;

                        // 檢查是否違反 Aliasing XOR Mutability (不能同時有多個可變借用或可變與不可變共存)
                        for loans in live_loans.values() {
                            for existing in loans {
                                if existing.place.overlaps(borrowed_place) {
                                    let is_conflict = matches!(
                                        (existing.kind, kind),
                                        (BorrowKind::Mut { .. }, _) | (_, BorrowKind::Mut { .. })
                                    );
                                    if is_conflict {
                                        report.borrow_conflicts.push((
                                            loc,
                                            borrowed_place.clone(),
                                            format!(
                                                "E0502: 無法以 {:?} 借用 `{}`，因為已存在衝突的借用 (Loan #{:?} 於 {})",
                                                kind, borrowed_place, existing.id, existing.issued_at
                                            ),
                                        ));
                                    }
                                }
                            }
                        }

                        report.active_loans.push(loan.clone());
                        live_loans.entry(loc).or_default().push(loan);
                    }
                }
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mir_cf_graph_construction_and_drop_elaboration() {
        let mut body = MirBody::new(1);
        let ret = body.add_local(MirType::Int(32), true, Span::new(0, 10), Some("_0".into()));
        let arg1 = body.add_local(
            MirType::Adt {
                name: "Pair".into(),
                fields: vec![
                    ("x".into(), MirType::Int(32)),
                    ("y".into(), MirType::Int(32)),
                ],
            },
            false,
            Span::new(10, 20),
            Some("_1".into()),
        );

        let mut bb0 = BasicBlockData::new(Some(Terminator {
            kind: TerminatorKind::Return,
            span: Span::new(20, 25),
        }));

        bb0.statements.push(Statement {
            kind: StatementKind::Assign(
                Place::from_local(ret),
                Rvalue::Use(Operand::Copy(Place::from_local(arg1).field(0))),
            ),
            span: Span::new(12, 18),
        });

        let bb0_id = body.add_block(bb0);
        assert_eq!(bb0_id, BasicBlock(0));
        assert_eq!(body.num_locals(), 2);

        // 測試 Drop Elaboration: 字段按正序 Drop
        let seq = DropElaborator::elaborate_scope_drops(&[arg1], &body);
        assert_eq!(seq.drops.len(), 2);
        assert_eq!(seq.drops[0].0, Place::from_local(arg1).field(0));
        assert_eq!(seq.drops[1].0, Place::from_local(arg1).field(1));
    }

    #[test]
    fn test_mir_move_analysis_and_borrowck() {
        let mut body = MirBody::new(0);
        let x = body.add_local(MirType::Int(32), true, Span::new(0, 5), Some("x".into()));
        let r1 = body.add_local(
            MirType::Ref(
                RegionVid(0),
                Box::new(MirType::Int(32)),
                BorrowKind::Mut {
                    allow_two_phase_borrow: false,
                },
            ),
            false,
            Span::new(5, 10),
            Some("r1".into()),
        );

        let mut bb0 = BasicBlockData::new(Some(Terminator {
            kind: TerminatorKind::Return,
            span: Span::new(30, 35),
        }));

        // 1. x = 42
        bb0.statements.push(Statement {
            kind: StatementKind::Assign(
                Place::from_local(x),
                Rvalue::Use(Operand::Constant(ConstantKind::Int(42))),
            ),
            span: Span::new(0, 5),
        });

        // 2. r1 = &mut x
        bb0.statements.push(Statement {
            kind: StatementKind::Assign(
                Place::from_local(r1),
                Rvalue::Ref(
                    RegionVid(0),
                    BorrowKind::Mut {
                        allow_two_phase_borrow: false,
                    },
                    Place::from_local(x),
                ),
            ),
            span: Span::new(6, 12),
        });

        // 3. x = 100 (衝突寫入！)
        bb0.statements.push(Statement {
            kind: StatementKind::Assign(
                Place::from_local(x),
                Rvalue::Use(Operand::Constant(ConstantKind::Int(100))),
            ),
            span: Span::new(15, 20),
        });

        body.add_block(bb0);

        let report = MirBorrowChecker::check_body(&body);
        assert_eq!(report.borrow_conflicts.len(), 1);
        assert!(report.borrow_conflicts[0].2.contains("E0506"));
    }
}
