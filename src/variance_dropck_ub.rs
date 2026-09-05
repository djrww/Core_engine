//! §8.4 型別協變/逆變/不變 (Variance)、Dropck 針眼法則與未定義行為 (UB) 核驗預言機 (UCG / Rustonomicon 標準)。
//!
//! 融匯 Rust 官方文檔與社群權威規範：
//!   1. **Variance 推導引擎 (Rust for Rustaceans / Crust of Rust 規範)**：
//!      - `&'a T` : 對 `'a` 協變 (Covariant)，對 `T` 協變
//!      - `&'a mut T` : 對 `'a` 協變，對 `T` 不變 (Invariant) —— 防止長壽命引用被短壽命覆蓋
//!      - `fn(T) -> U` : 對 `T` 逆變 (Contravariant)，對 `U` 協變
//!      - `UnsafeCell<T>` / `*mut T` : 對 `T` 不變
//!   2. **Dropck 與針眼法則 (Eye-of-the-Needle / `#[may_dangle]`)**：
//!      - 當結構體實現 `Drop` 時，其泛型參數與生命週期必須嚴格存活於實例生命期之外；
//!      - `PhantomData<T>` (擁有權) vs `PhantomData<*const T>` (非擁有引用)；
//!      - `#[may_dangle]` 逃生艙口合規性核驗。
//!   3. **未定義行為 (Undefined Behavior) 診斷預言機 (UCG / The Rust Reference UB 章)**：
//!      - Stacked Borrows / Tree Borrows 別名規則違反
//!      - 未初始化記憶體位元讀取 (Invalid bool / enum discriminant)
//!      - 懸掛指標 / 奇異指標解引用與對齊 (Alignment & Null Dereference)
//!      - 跨 FFI 邊界 Unwind 逃逸。

use crate::mir::{BorrowKind, MirType};
use std::fmt::{self, Display, Formatter};

/// 型別變異性 (Variance)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variance {
    Covariant,     // 協變: 若 T <: U 則 F<T> <: F<U>
    Contravariant, // 逆變: 若 T <: U 則 F<U> <: F<T>
    Invariant,     // 不變: 僅當 T == U 時 F<T> == F<U>
    Bivariant,     // 雙變/無關 (如未使用的泛型參數)
}

impl Display for Variance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Variance::Covariant => write!(f, "Covariant (+)"),
            Variance::Contravariant => write!(f, "Contravariant (-)"),
            Variance::Invariant => write!(f, "Invariant (*)"),
            Variance::Bivariant => write!(f, "Bivariant (0)"),
        }
    }
}

impl Variance {
    /// 變異性乘法複合規律: v1 * v2
    pub fn compose(self, other: Variance) -> Variance {
        match (self, other) {
            (Variance::Bivariant, _) | (_, Variance::Bivariant) => Variance::Bivariant,
            (Variance::Invariant, _) | (_, Variance::Invariant) => Variance::Invariant,
            (Variance::Covariant, v) | (v, Variance::Covariant) => v,
            (Variance::Contravariant, Variance::Contravariant) => Variance::Covariant,
        }
    }

    /// 結合律合併 (Join / Least Upper Bound in Variance Lattice)
    pub fn join(self, other: Variance) -> Variance {
        match (self, other) {
            (Variance::Bivariant, v) | (v, Variance::Bivariant) => v,
            (v1, v2) if v1 == v2 => v1,
            _ => Variance::Invariant,
        }
    }
}

/// 泛型型別建構子的變異性分析引擎
pub struct VarianceEngine;

impl VarianceEngine {
    /// 分析特定型別對特定泛型型別參數 `param_name` (如 "T") 的變異性
    pub fn infer_variance_of_param(ty: &MirType, param_name: &str) -> Variance {
        match ty {
            MirType::TypeParam(name) if name == param_name => Variance::Covariant,
            MirType::LifetimeParam(name) if name == param_name => Variance::Covariant,
            MirType::Bool
            | MirType::Int(_)
            | MirType::Uint(_)
            | MirType::Never
            | MirType::TypeParam(_)
            | MirType::LifetimeParam(_) => Variance::Bivariant,

            MirType::Tuple(fields) => {
                let mut v = Variance::Bivariant;
                for f in fields {
                    v = v.join(Self::infer_variance_of_param(f, param_name));
                }
                v
            }

            MirType::Ref(_, inner, kind) => match kind {
                BorrowKind::Shared => {
                    // &'a T 對 T 協變
                    Variance::Covariant.compose(Self::infer_variance_of_param(inner, param_name))
                }
                BorrowKind::Mut { .. } => {
                    // &'a mut T 對 T 不變 (Invariant) 若 T 出現
                    let inner_v = Self::infer_variance_of_param(inner, param_name);
                    if inner_v != Variance::Bivariant {
                        Variance::Invariant
                    } else {
                        Variance::Bivariant
                    }
                }
                _ => {
                    let inner_v = Self::infer_variance_of_param(inner, param_name);
                    if inner_v != Variance::Bivariant {
                        Variance::Invariant
                    } else {
                        Variance::Bivariant
                    }
                }
            },

            MirType::RawPtr(inner, is_mut) => {
                let inner_v = Self::infer_variance_of_param(inner, param_name);
                if *is_mut {
                    // *mut T 對 T 不變
                    if inner_v != Variance::Bivariant {
                        Variance::Invariant
                    } else {
                        Variance::Bivariant
                    }
                } else {
                    // *const T 對 T 協變
                    Variance::Covariant.compose(inner_v)
                }
            }

            MirType::Array(inner, _) | MirType::Slice(inner) => {
                Variance::Covariant.compose(Self::infer_variance_of_param(inner, param_name))
            }

            MirType::FnPtr { params, ret } => {
                let mut v = Variance::Bivariant;
                for p in params {
                    // 函數參數為逆變 (Contravariant)
                    let p_v = Self::infer_variance_of_param(p, param_name);
                    v = v.join(Variance::Contravariant.compose(p_v));
                }
                let ret_v = Self::infer_variance_of_param(ret, param_name);
                v = v.join(Variance::Covariant.compose(ret_v));
                v
            }

            MirType::Adt { fields, .. } => {
                let mut v = Variance::Bivariant;
                for (_, f_ty) in fields {
                    v = v.join(Self::infer_variance_of_param(f_ty, param_name));
                }
                v
            }
        }
    }
}

// =========================================================================
// 2. Dropck 針眼法則核驗器 (Dropck Eye-of-the-Needle Checker)
// =========================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropckGenericConstraint {
    pub type_param: String,
    pub has_may_dangle: bool,
    pub used_in_destructor: bool,
}

pub struct DropckChecker;

impl DropckChecker {
    /// 驗證 Drop 實現是否滿足 Dropck 安全性
    /// 規則：若型別參數 T 沒有 `#[may_dangle]` 標籤，則 T 必須嚴格存活長於容器；
    /// 若標有 `#[may_dangle]`，則析構函數內絕對不得解引用或讀取 T 的資料。
    pub fn verify_dropck_safety(
        struct_name: &str,
        constraints: &[DropckGenericConstraint],
    ) -> Result<(), String> {
        for c in constraints {
            if c.has_may_dangle && c.used_in_destructor {
                return Err(format!(
                    "Dropck 違反 (針眼法則): 結構體 `{}` 的泛型參數 `{}` 標有 `#[may_dangle]`，但在 drop() 內被非法存取/解引用！",
                    struct_name, c.type_param
                ));
            }
        }
        Ok(())
    }
}

// =========================================================================
// 3. 未定義行為 (Undefined Behavior) 診斷預言機 (UCG / Reference UB)
// =========================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UbViolation {
    AliasingViolation {
        description: String,
        target_place: String,
    },
    InvalidBitPattern {
        type_name: String,
        raw_value: u64,
        reason: String,
    },
    NullOrDanglingDereference {
        address: u64,
    },
    MisalignedAccess {
        address: u64,
        required_alignment: usize,
    },
    DataRace {
        location: String,
        conflicting_threads: (u32, u32),
    },
    UnwindAcrossFfiBoundary {
        extern_fn: String,
    },
}

impl Display for UbViolation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UbViolation::AliasingViolation {
                description,
                target_place,
            } => {
                write!(
                    f,
                    "UB: 別名規則違反 (Stacked Borrows) - {} 在 `{}`",
                    description, target_place
                )
            }
            UbViolation::InvalidBitPattern {
                type_name,
                raw_value,
                reason,
            } => {
                write!(
                    f,
                    "UB: 無效位元模式 - 型別 `{}` 讀取到非法定值 0x{:x} ({})",
                    type_name, raw_value, reason
                )
            }
            UbViolation::NullOrDanglingDereference { address } => {
                write!(f, "UB: 解引用空指標或懸掛指標 0x{:x}", address)
            }
            UbViolation::MisalignedAccess {
                address,
                required_alignment,
            } => {
                write!(
                    f,
                    "UB: 未對齊記憶體存取 0x{:x} (需對齊到 {} 位元組)",
                    address, required_alignment
                )
            }
            UbViolation::DataRace {
                location,
                conflicting_threads,
            } => {
                write!(
                    f,
                    "UB: 數據競爭 (Data Race) 在 {}，線程 #{} 與 #{} 衝突",
                    location, conflicting_threads.0, conflicting_threads.1
                )
            }
            UbViolation::UnwindAcrossFfiBoundary { extern_fn } => {
                write!(
                    f,
                    "UB: 異常 Unwind 跨越了無 C-unwind 說明的 FFI 邊界 `{}`",
                    extern_fn
                )
            }
        }
    }
}

pub struct UbDiagnosticOracle;

impl UbDiagnosticOracle {
    /// 檢查布林型別位元合法性 (必須為 0 或 1)
    pub fn check_bool_validity(raw_byte: u8) -> Option<UbViolation> {
        if raw_byte > 1 {
            Some(UbViolation::InvalidBitPattern {
                type_name: "bool".into(),
                raw_value: raw_byte as u64,
                reason: "bool 型別之底層位元只能為 0 (false) 或 1 (true)".into(),
            })
        } else {
            None
        }
    }

    /// 檢查指標對齊與非空性
    pub fn check_pointer_access(addr: u64, align: usize) -> Option<UbViolation> {
        if addr == 0 {
            return Some(UbViolation::NullOrDanglingDereference { address: 0 });
        }
        if align > 1 && !addr.is_multiple_of(align as u64) {
            return Some(UbViolation::MisalignedAccess {
                address: addr,
                required_alignment: align,
            });
        }
        None
    }

    /// 檢查 Stacked Borrows 別名讀寫合法性
    pub fn check_stacked_borrows_access(
        is_write: bool,
        has_active_unique: bool,
        active_shared_readers: usize,
    ) -> Option<UbViolation> {
        if is_write && active_shared_readers > 0 {
            Some(UbViolation::AliasingViolation {
                description: "在存在活躍共享引用 (&T) 的同時發起寫入".into(),
                target_place: "*ptr".into(),
            })
        } else if is_write && !has_active_unique {
            Some(UbViolation::AliasingViolation {
                description: "在缺乏 Unique Tag 權限的情況下寫入記憶體".into(),
                target_place: "*ptr".into(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variance_rules() {
        let t_param = MirType::TypeParam("T".into());

        // &'a T: 對 T 協變 (Covariant)
        let shared_ref = MirType::Ref(
            crate::mir::RegionVid(0),
            Box::new(t_param.clone()),
            BorrowKind::Shared,
        );
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&shared_ref, "T"),
            Variance::Covariant
        );

        // &'a mut T: 對 T 不變 (Invariant)
        let mut_ref = MirType::Ref(
            crate::mir::RegionVid(0),
            Box::new(t_param.clone()),
            BorrowKind::Mut {
                allow_two_phase_borrow: false,
            },
        );
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&mut_ref, "T"),
            Variance::Invariant
        );

        // fn(T) -> (): 對 T 逆變 (Contravariant)
        let fn_ty = MirType::FnPtr {
            params: vec![t_param.clone()],
            ret: Box::new(MirType::Tuple(vec![])),
        };
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&fn_ty, "T"),
            Variance::Contravariant
        );

        // *const T: 對 T 協變
        let const_ptr = MirType::RawPtr(Box::new(t_param.clone()), false);
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&const_ptr, "T"),
            Variance::Covariant
        );

        // *mut T: 對 T 不變
        let mut_ptr = MirType::RawPtr(Box::new(t_param.clone()), true);
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&mut_ptr, "T"),
            Variance::Invariant
        );
    }

    #[test]
    fn test_dropck_may_dangle_safety() {
        // 安全案例: #[may_dangle] 且未在 destructor 內使用
        let safe_c = vec![DropckGenericConstraint {
            type_param: "T".into(),
            has_may_dangle: true,
            used_in_destructor: false,
        }];
        assert!(DropckChecker::verify_dropck_safety("CustomBox", &safe_c).is_ok());

        // 不安全案例: #[may_dangle] 但在 destructor 內非法存取
        let unsafe_c = vec![DropckGenericConstraint {
            type_param: "T".into(),
            has_may_dangle: true,
            used_in_destructor: true,
        }];
        assert!(DropckChecker::verify_dropck_safety("BadBox", &unsafe_c).is_err());
    }

    #[test]
    fn test_ub_oracle_diagnostics() {
        // 1. 布林非法位元值 2 觸發 UB
        assert!(UbDiagnosticOracle::check_bool_validity(2).is_some());
        assert!(UbDiagnosticOracle::check_bool_validity(0).is_none());
        assert!(UbDiagnosticOracle::check_bool_validity(1).is_none());

        // 2. 空指標解引用
        assert_eq!(
            UbDiagnosticOracle::check_pointer_access(0, 4),
            Some(UbViolation::NullOrDanglingDereference { address: 0 })
        );

        // 3. 未對齊存取 (位址 0x1001 存取 4 位元組對齊)
        assert_eq!(
            UbDiagnosticOracle::check_pointer_access(0x1001, 4),
            Some(UbViolation::MisalignedAccess {
                address: 0x1001,
                required_alignment: 4
            })
        );

        // 4. Stacked Borrows 衝突 (有讀者時發起寫入)
        assert!(UbDiagnosticOracle::check_stacked_borrows_access(true, true, 2).is_some());
    }

    #[test]
    fn test_variance_lattice_compose_join_and_display() {
        use Variance::*;
        // compose 乘法複合:吸收律(0 吸收一切)、(*) 吸收除 0 外一切。
        assert_eq!(Bivariant.compose(Invariant), Bivariant);
        assert_eq!(Covariant.compose(Bivariant), Bivariant);
        assert_eq!(Invariant.compose(Covariant), Invariant);
        assert_eq!(Covariant.compose(Contravariant), Contravariant);
        assert_eq!(Contravariant.compose(Covariant), Contravariant);
        assert_eq!(Contravariant.compose(Contravariant), Covariant);
        // join 格 LUB:0 為單位元、同值冪等、異值取 (*)。
        assert_eq!(Bivariant.join(Invariant), Invariant);
        assert_eq!(Covariant.join(Bivariant), Covariant);
        assert_eq!(Covariant.join(Covariant), Covariant);
        assert_eq!(Covariant.join(Contravariant), Invariant);
        assert_eq!(Invariant.join(Contravariant), Invariant);
        // Display 四態
        let text = format!("{Covariant} {Contravariant} {Invariant} {Bivariant}");
        assert_eq!(
            text,
            "Covariant (+) Contravariant (-) Invariant (*) Bivariant (0)"
        );
    }

    #[test]
    fn test_infer_variance_exhaustive_branches() {
        use Variance::*;
        let t = MirType::TypeParam("T".into());
        // 生命週期參數命中 → 協變;未命中 → 雙變。
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&MirType::LifetimeParam("'a".into()), "'a"),
            Covariant
        );
        // 基礎型別不含 T → 雙變。
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&MirType::Bool, "T"),
            Bivariant
        );
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&MirType::Never, "T"),
            Bivariant
        );
        // Tuple:join 合併。
        assert_eq!(
            VarianceEngine::infer_variance_of_param(
                &MirType::Tuple(vec![t.clone(), MirType::Bool]),
                "T"
            ),
            Covariant
        );
        // &'a mut u8(內層雙變) → 雙變;*mut T → 不變;*const T → 協變。
        let mut_no_t = MirType::Ref(
            crate::mir::RegionVid(0),
            Box::new(MirType::Uint(8)),
            BorrowKind::Mut {
                allow_two_phase_borrow: false,
            },
        );
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&mut_no_t, "T"),
            Bivariant
        );
        assert_eq!(
            VarianceEngine::infer_variance_of_param(
                &MirType::RawPtr(Box::new(t.clone()), true),
                "T"
            ),
            Invariant
        );
        assert_eq!(
            VarianceEngine::infer_variance_of_param(
                &MirType::RawPtr(Box::new(t.clone()), false),
                "T"
            ),
            Covariant
        );
        // Array/Slice 對元素協變。
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&MirType::Array(Box::new(t.clone()), 3), "T"),
            Covariant
        );
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&MirType::Slice(Box::new(t.clone())), "T"),
            Covariant
        );
        // FnPtr:回傳位置協變。
        let fn_ret_t = MirType::FnPtr {
            params: vec![MirType::Bool],
            ret: Box::new(t.clone()),
        };
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&fn_ret_t, "T"),
            Covariant
        );
        // Adt 欄位 join。
        let adt = MirType::Adt {
            name: "Box".into(),
            fields: vec![("0".into(), t.clone())],
        };
        assert_eq!(
            VarianceEngine::infer_variance_of_param(&adt, "T"),
            Covariant
        );
    }

    #[test]
    fn test_ub_violation_display_all_six_families() {
        let cases = [
            UbViolation::AliasingViolation {
                description: "共享引用存活時寫入".into(),
                target_place: "*r".into(),
            },
            UbViolation::InvalidBitPattern {
                type_name: "bool".into(),
                raw_value: 2,
                reason: "僅允許 0/1".into(),
            },
            UbViolation::NullOrDanglingDereference { address: 0xdead },
            UbViolation::MisalignedAccess {
                address: 0x1001,
                required_alignment: 8,
            },
            UbViolation::DataRace {
                location: "x".into(),
                conflicting_threads: (3, 7),
            },
            UbViolation::UnwindAcrossFfiBoundary {
                extern_fn: "c_abort".into(),
            },
        ];
        for v in &cases {
            let s = v.to_string();
            assert!(s.starts_with("UB: "), "Display 應以 UB: 開頭: {s}");
            assert!(!s.ends_with(' '), "Display 不應以空白結尾: {s}");
        }
        // 六家族逐一關鍵詞
        assert!(cases[0].to_string().contains("別名規則違反"));
        assert!(cases[1].to_string().contains("無效位元模式"));
        assert!(cases[2].to_string().contains("0xdead"));
        assert!(cases[3].to_string().contains("未對齊"));
        assert!(cases[4].to_string().contains("#3 與 #7"));
        assert!(cases[5].to_string().contains("c_abort"));
    }

    #[test]
    fn test_stacked_borrows_oracle_three_outcomes() {
        // 寫入 + 缺 Unique Tag(無共享讀者) → 別名違反。
        let no_unique = UbDiagnosticOracle::check_stacked_borrows_access(true, false, 0);
        assert!(matches!(
            no_unique,
            Some(UbViolation::AliasingViolation { .. })
        ));
        // 讀取永遠合法(即使無 Unique、有共享讀者)。
        assert_eq!(
            UbDiagnosticOracle::check_stacked_borrows_access(false, false, 3),
            None
        );
        // 合法寫入:有 Unique 且無共享讀者。
        assert_eq!(
            UbDiagnosticOracle::check_stacked_borrows_access(true, true, 0),
            None
        );
        // 對齊良好且非空 → None。
        assert_eq!(UbDiagnosticOracle::check_pointer_access(0x1000, 8), None);
        assert_eq!(UbDiagnosticOracle::check_pointer_access(7, 1), None);
    }
}
