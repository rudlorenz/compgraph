use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub enum PtrCompNode {
    Input {
        name: &'static str,
        value: Rc<Cell<Option<f32>>>,
        deps: Rc<RefCell<Vec<Weak<Self>>>>,
    },
    Constant {
        value: f32,
    },
    BinaryOp {
        rhs: Rc<Self>,
        lhs: Rc<Self>,
        cache: Cell<Option<f32>>,
        op_type: BinaryOpType,
    },
    UnaryOp {
        arg: Rc<Self>,
        cache: Cell<Option<f32>>,
        op_type: UnaryOpType,
    },
}

pub type CompNode = Rc<PtrCompNode>;

impl PtrCompNode {
    /// Add link to dependent node to correlating `BoxedCompNode::Input`.
    fn add_dependency_link(&self, dep: Weak<Self>) {
        match self {
            Self::Input { deps, .. } => deps.borrow_mut().push(dep),

            Self::BinaryOp { lhs, rhs, .. } => {
                lhs.add_dependency_link(dep.clone());
                rhs.add_dependency_link(dep);
            }

            Self::UnaryOp { arg, .. } => arg.add_dependency_link(dep),

            Self::Constant { .. } => (),
        }
    }

    /// Set value for `BoxedCompNode`.
    pub fn set(&self, val: f32) {
        if let Self::Input { value, deps, .. } = self {
            value.set(Some(val));

            for dep in deps.borrow().iter() {
                if let Some(dep) = dep.upgrade() {
                    dep.invalidate_cache();
                }
            }
        }
    }

    fn invalidate_cache(&self) {
        match self {
            Self::BinaryOp { cache, .. } | Self::UnaryOp { cache, .. } => {
                log::debug!("Invalidate cache {self}");
                cache.set(None);
            }

            Self::Input { .. } | Self::Constant { .. } => (),
        }
    }

    pub fn get_cache_or_value(&self) -> Option<f32> {
        match self {
            Self::Input { value, .. } => value.get(),
            Self::Constant { value } => Some(*value),
            Self::BinaryOp { cache, .. } | Self::UnaryOp { cache, .. } => cache.get(),
        }
    }

    /// Returns the compute of this [`BoxedCompNode`].
    ///
    /// # Panics
    ///
    /// Panics if one of the used inputs value is not set.
    pub fn compute(&self) -> f32 {
        match self {
            Self::Constant { value } => *value,

            Self::Input {
                name,
                value,
                deps: _,
            } => value
                .get()
                .unwrap_or_else(|| panic!("Input {name} value not set. Aborting compute")),

            Self::BinaryOp {
                lhs,
                rhs,
                cache,
                op_type,
            } => cache.get().map_or_else(
                || {
                    log::debug!("Cache miss {self}");
                    let result = match op_type {
                        BinaryOpType::Sum => lhs.compute() + rhs.compute(),
                        BinaryOpType::Mul => lhs.compute() * rhs.compute(),
                        BinaryOpType::Sub => lhs.compute() - rhs.compute(),
                        BinaryOpType::Div => lhs.compute() / rhs.compute(),
                        BinaryOpType::Pow => lhs.compute().powf(rhs.compute()),
                    };
                    cache.set(Some(result));
                    result
                },
                |cached_value| cached_value,
            ),
            Self::UnaryOp {
                arg,
                cache,
                op_type,
            } => cache.get().map_or_else(
                || {
                    log::debug!("Cache miss {self}");
                    let result = match op_type {
                        UnaryOpType::Sin => arg.compute().sin(),
                        UnaryOpType::Cos => arg.compute().cos(),
                    };
                    cache.set(Some(result));
                    result
                },
                |cached_value| cached_value,
            ),
        }
    }
}

impl std::fmt::Display for PtrCompNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant { value } => write!(f, "{value}"),
            Self::Input {
                name,
                value: _,
                deps: _,
            } => write!(f, "{name}"),
            Self::BinaryOp {
                rhs, lhs, op_type, ..
            } => match op_type {
                BinaryOpType::Sum => write!(f, "({lhs} + {rhs})"),
                BinaryOpType::Mul => write!(f, "({lhs} * {rhs})"),
                BinaryOpType::Sub => write!(f, "({lhs} - {rhs})"),
                BinaryOpType::Div => write!(f, "({lhs} / {rhs})"),
                BinaryOpType::Pow => write!(f, "{lhs}^{rhs}"),
            },
            Self::UnaryOp { arg, op_type, .. } => match op_type {
                UnaryOpType::Sin => write!(f, "sin({arg})"),
                UnaryOpType::Cos => write!(f, "cos({arg})"),
            },
        }
    }
}

impl std::fmt::Debug for PtrCompNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant { value } => f.debug_struct("Constant").field("value", value).finish(),
            Self::Input { name, value, deps } => f
                .debug_struct("Input")
                .field("name", name)
                .field("value", value)
                .field(
                    "deps",
                    &deps
                        .borrow()
                        .iter()
                        .filter_map(Weak::upgrade)
                        .map(|item| format!("{item}"))
                        .collect::<Vec<String>>()
                        .join(", "),
                )
                .finish(),
            Self::BinaryOp {
                rhs,
                lhs,
                cache,
                op_type,
            } => f
                .debug_struct(&format!("{op_type}"))
                .field("rhs", rhs)
                .field("lhs", lhs)
                .field("cache", cache)
                .finish(),
            Self::UnaryOp {
                arg,
                cache,
                op_type,
            } => f
                .debug_struct(&format!("{op_type}"))
                .field("arg", arg)
                .field("cache", cache)
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOpType {
    Sum,
    Mul,
    Sub,
    Div,
    Pow,
}

impl std::fmt::Display for BinaryOpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sum => write!(f, "sum"),
            Self::Mul => write!(f, "mul"),
            Self::Sub => write!(f, "sub"),
            Self::Div => write!(f, "div"),
            Self::Pow => write!(f, "pow"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOpType {
    Sin,
    Cos,
}

impl std::fmt::Display for UnaryOpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sin => write!(f, "sin"),
            Self::Cos => write!(f, "cos"),
        }
    }
}

/// Create empty input with given name.
#[must_use]
pub fn create_input(name: &'static str) -> CompNode {
    Rc::new(PtrCompNode::Input {
        name,
        value: Rc::new(Cell::new(None)),
        deps: Rc::new(RefCell::new(Vec::new())),
    })
}

/// Create empty input with given name and given value.
#[must_use]
pub fn create_input_from(name: &'static str, value: f32) -> CompNode {
    Rc::new(PtrCompNode::Input {
        name,
        value: Rc::new(Cell::new(Some(value))),
        deps: Rc::new(RefCell::new(Vec::new())),
    })
}

#[must_use]
pub fn sum(lhs: CompNode, rhs: CompNode) -> CompNode {
    let cache = lhs
        .get_cache_or_value()
        .zip(rhs.get_cache_or_value())
        .map(|(lhs_cache, rhs_cache)| lhs_cache + rhs_cache);

    let result = Rc::new(PtrCompNode::BinaryOp {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
        cache: Cell::new(cache),
        op_type: BinaryOpType::Sum,
    });

    lhs.add_dependency_link(Rc::downgrade(&result));
    rhs.add_dependency_link(Rc::downgrade(&result));

    result
}

#[must_use]
pub fn mul(lhs: CompNode, rhs: CompNode) -> CompNode {
    let cache = lhs
        .get_cache_or_value()
        .zip(rhs.get_cache_or_value())
        .map(|(lhs_cache, rhs_cache)| lhs_cache * rhs_cache);
    let result = Rc::new(PtrCompNode::BinaryOp {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
        cache: Cell::new(cache),
        op_type: BinaryOpType::Mul,
    });

    lhs.add_dependency_link(Rc::downgrade(&result));
    rhs.add_dependency_link(Rc::downgrade(&result));

    result
}

#[must_use]
pub fn sub(lhs: CompNode, rhs: CompNode) -> CompNode {
    let cache = lhs
        .get_cache_or_value()
        .zip(rhs.get_cache_or_value())
        .map(|(lhs_cache, rhs_cache)| lhs_cache - rhs_cache);
    let result = Rc::new(PtrCompNode::BinaryOp {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
        cache: Cell::new(cache),
        op_type: BinaryOpType::Sub,
    });

    lhs.add_dependency_link(Rc::downgrade(&result));
    rhs.add_dependency_link(Rc::downgrade(&result));

    result
}

#[must_use]
pub fn div(lhs: CompNode, rhs: CompNode) -> CompNode {
    let cache = lhs
        .get_cache_or_value()
        .zip(rhs.get_cache_or_value())
        .map(|(lhs_cache, rhs_cache)| lhs_cache / rhs_cache);
    let result = Rc::new(PtrCompNode::BinaryOp {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
        cache: Cell::new(cache),
        op_type: BinaryOpType::Div,
    });

    lhs.add_dependency_link(Rc::downgrade(&result));
    rhs.add_dependency_link(Rc::downgrade(&result));

    result
}

#[must_use]
pub fn sin(arg: CompNode) -> CompNode {
    let cache = arg.get_cache_or_value().map(f32::sin);
    let result = Rc::new(PtrCompNode::UnaryOp {
        arg: arg.clone(),
        cache: Cell::new(cache),
        op_type: UnaryOpType::Sin,
    });

    arg.add_dependency_link(Rc::downgrade(&result));

    result
}

#[must_use]
pub fn cos(arg: CompNode) -> CompNode {
    let cache = arg.get_cache_or_value().map(f32::cos);
    let result = Rc::new(PtrCompNode::UnaryOp {
        arg: arg.clone(),
        cache: Cell::new(cache),
        op_type: UnaryOpType::Cos,
    });

    arg.add_dependency_link(Rc::downgrade(&result));

    result
}

#[must_use]
pub fn powf(arg: CompNode, power: CompNode) -> CompNode {
    let cache = arg
        .get_cache_or_value()
        .zip(power.get_cache_or_value())
        .map(|(arg_cache, power_cache)| arg_cache.powf(power_cache));

    let result = Rc::new(PtrCompNode::BinaryOp {
        lhs: arg.clone(),
        rhs: power.clone(),
        cache: Cell::new(cache),
        op_type: BinaryOpType::Pow,
    });

    arg.add_dependency_link(Rc::downgrade(&result));
    power.add_dependency_link(Rc::downgrade(&result));

    result
}

#[must_use]
pub fn pow(arg: CompNode, power: f32) -> CompNode {
    let cache = arg
        .get_cache_or_value()
        .map(|arg_cache| arg_cache.powf(power));
    let result = Rc::new(PtrCompNode::BinaryOp {
        lhs: arg.clone(),
        rhs: Rc::new(PtrCompNode::Constant { value: power }),
        cache: Cell::new(cache),
        op_type: BinaryOpType::Pow,
    });

    arg.add_dependency_link(Rc::downgrade(&result));

    result
}

#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
fn value_generator() -> impl Strategy<Value = (f32, f32)> {
    (prop::num::f32::NORMAL, prop::num::f32::NORMAL)
}

#[cfg(test)]
proptest! {
#![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn test_set((gen_a, _) in value_generator()) {
        let a = create_input("a");
        a.set(gen_a);

        assert_eq!(gen_a, a.compute());
    }

    #[test]
    fn test_create_set((gen_a, _) in value_generator()) {
        let a = create_input_from("a", gen_a);

        assert_eq!(gen_a, a.compute());
    }

    #[test]
    fn test_sum((gen_a, gen_b) in value_generator()) {
        let a = create_input_from("a", gen_a);
        let b = create_input_from("b", gen_b);
        let expr = sum(a, b);

        assert_eq!(gen_a + gen_b, expr.compute())
    }

    #[test]
    fn test_mul((gen_a, gen_b) in value_generator()) {
        let a = create_input_from("a", gen_a);
        let b = create_input_from("b", gen_b);
        let expr = mul(a, b);

        assert_eq!(gen_a * gen_b, expr.compute())
    }

    #[test]
    fn test_sub((gen_a, gen_b) in value_generator()) {
        let a = create_input_from("a", gen_a);
        let b = create_input_from("b", gen_b);
        let expr = sub(a, b);

        assert_eq!(gen_a - gen_b, expr.compute())
    }

    #[test]
    fn test_div((gen_a, gen_b) in value_generator()) {
        let a = create_input_from("a", gen_a);
        let b = create_input_from("b", gen_b);
        let expr = div(a, b);

        assert_eq!(gen_a / gen_b, expr.compute())
    }

    #[test]
    fn test_sin((gen_a, _) in value_generator()) {
        let arg = create_input_from("a", gen_a);

        let expr = sin(arg);

        assert_eq!(f32::sin(gen_a), expr.compute())
    }

    #[test]
    fn test_cos((gen_a, _) in value_generator()) {
        let arg = create_input_from("a", gen_a);

        let expr = cos(arg);

        assert_eq!(f32::cos(gen_a), expr.compute())
    }

    #[test]
    fn test_pow((gen_a, gen_b) in (-1000..1000, -1000..1000)) {
        let arg = create_input_from("a", gen_a as f32);

        let expr = pow(arg, gen_b as f32);

        assert_eq!(f32::powf(gen_a as f32, gen_b as f32), expr.compute())
    }

    #[test]
    fn test_powf((gen_a, gen_b) in (-1000..1000, -1000..1000)) {
        let arg = create_input_from("a", gen_a as f32);
        let power = create_input_from("pw", gen_b as f32);

        let expr = powf(arg, power);

        assert_eq!(f32::powf(gen_a as f32, gen_b as f32), expr.compute())
    }

    // TODO: add compound tests sum(mul()), div(sum) etc
}
