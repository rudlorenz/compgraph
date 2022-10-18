use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub enum BoxedCompNode {
    Input {
        name: &'static str,
        value: Rc<Cell<Option<f32>>>,
        deps: Rc<RefCell<Vec<Weak<BoxedCompNode>>>>,
    },
    Sum {
        lhs: Rc<BoxedCompNode>,
        rhs: Rc<BoxedCompNode>,
        cache: Cell<Option<f32>>,
    },
    Mul {
        lhs: Rc<BoxedCompNode>,
        rhs: Rc<BoxedCompNode>,
        cache: Cell<Option<f32>>,
    },
    Sub {
        lhs: Rc<BoxedCompNode>,
        rhs: Rc<BoxedCompNode>,
        cache: Cell<Option<f32>>,
    },
    Div {
        lhs: Rc<BoxedCompNode>,
        rhs: Rc<BoxedCompNode>,
        cache: Cell<Option<f32>>,
    },
    // ✝️️
    Sin {
        arg: Rc<BoxedCompNode>,
        cache: Cell<Option<f32>>,
    },
    // ⸸
    Cos {
        arg: Rc<BoxedCompNode>,
        cache: Cell<Option<f32>>,
    },
    Pow {
        arg: Rc<BoxedCompNode>,
        power: Rc<BoxedCompNode>,
        cache: Cell<Option<f32>>,
    },
}

pub type CompNode = Rc<BoxedCompNode>;

impl BoxedCompNode {
    fn update_deps(&self, dep: Weak<Self>) {
        match self {
            Self::Input { deps, .. } => deps.borrow_mut().push(dep),

            Self::Sum { lhs, rhs, .. }
            | Self::Mul { lhs, rhs, .. }
            | Self::Sub { lhs, rhs, .. }
            | Self::Div { lhs, rhs, .. }
            | Self::Pow {
                arg: lhs,
                power: rhs,
                ..
            } => {
                lhs.update_deps(dep.clone());
                rhs.update_deps(dep);
            }

            Self::Sin { arg, .. } | Self::Cos { arg, .. } => arg.update_deps(dep),
        }
    }

    pub fn set(&self, val: f32) {
        if let Self::Input {
            name: _,
            value,
            deps,
        } = self
        {
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
            Self::Sum { cache, .. }
            | Self::Mul { cache, .. }
            | Self::Sub { cache, .. }
            | Self::Div { cache, .. }
            | Self::Sin { cache, .. }
            | Self::Cos { cache, .. }
            | Self::Pow { cache, .. } => {
                log::debug!("Invalidate cache {self}");
                cache.set(None)
            }

            Self::Input { .. } => (),
        }
    }

    pub fn compute(&self) -> f32 {
        match self {
            Self::Input {
                name,
                value,
                deps: _,
            } => value
                .get()
                .unwrap_or_else(|| panic!("Input {name} value not set. Aborting compute")),

            Self::Sum { lhs, rhs, cache } => {
                if let Some(cached_value) = cache.get() {
                    cached_value
                } else {
                    log::debug!("Cache miss {}", self);
                    let result = lhs.compute() + rhs.compute();
                    cache.set(Some(result));
                    result
                }
            }
            Self::Mul { lhs, rhs, cache } => {
                if let Some(cached_value) = cache.get() {
                    cached_value
                } else {
                    log::debug!("Cache miss {}", self);
                    let result = lhs.compute() * rhs.compute();
                    cache.set(Some(result));
                    result
                }
            }
            Self::Sub { lhs, rhs, cache } => {
                if let Some(cached_value) = cache.get() {
                    cached_value
                } else {
                    log::debug!("Cache miss {}", self);
                    let result = lhs.compute() - rhs.compute();
                    cache.set(Some(result));
                    result
                }
            }
            Self::Div { lhs, rhs, cache } => {
                if let Some(cached_value) = cache.get() {
                    cached_value
                } else {
                    log::debug!("Cache miss {}", self);
                    let result = lhs.compute() / rhs.compute();
                    cache.set(Some(result));
                    result
                }
            }
            Self::Sin { arg, cache } => {
                if let Some(cached_value) = cache.get() {
                    cached_value
                } else {
                    log::debug!("Cache miss {}", self);
                    let result = f32::sin(arg.compute());
                    cache.set(Some(result));
                    result
                }
            }
            Self::Cos { arg, cache } => {
                if let Some(cached_value) = cache.get() {
                    cached_value
                } else {
                    log::debug!("Cache miss {}", self);
                    let result = f32::cos(arg.compute());
                    cache.set(Some(result));
                    result
                }
            }
            Self::Pow { arg, power, cache } => {
                if let Some(cached_value) = cache.get() {
                    cached_value
                } else {
                    log::debug!("Cache miss {}", self);
                    let result = f32::powf(arg.compute(), power.compute());
                    cache.set(Some(result));
                    result
                }
            }
        }
    }
}

impl std::fmt::Display for BoxedCompNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input {
                name,
                value: _,
                deps: _,
            } => write!(f, "{name}"),
            Self::Sum { lhs, rhs, .. } => write!(f, "({lhs} + {rhs})"),
            Self::Mul { lhs, rhs, .. } => write!(f, "({lhs} * {rhs})"),
            Self::Sub { lhs, rhs, .. } => write!(f, "({lhs} - {rhs})"),
            Self::Div { lhs, rhs, .. } => write!(f, "({lhs} / {rhs})"),
            Self::Sin { arg, .. } => write!(f, "sin({arg})"),
            Self::Cos { arg, .. } => write!(f, "cos({arg})"),
            Self::Pow { arg, power, .. } => write!(f, "{arg}^{power}"),
        }
    }
}

impl std::fmt::Debug for BoxedCompNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
            Self::Sum { lhs, rhs, cache } => f
                .debug_struct("Sum")
                .field("lhs", lhs)
                .field("rhs", rhs)
                .field("cache", cache)
                .finish(),
            Self::Mul { lhs, rhs, cache } => f
                .debug_struct("Mul")
                .field("lhs", lhs)
                .field("rhs", rhs)
                .field("cache", cache)
                .finish(),
            Self::Sub { lhs, rhs, cache } => f
                .debug_struct("Sub")
                .field("lhs", lhs)
                .field("rhs", rhs)
                .field("cache", cache)
                .finish(),
            Self::Div { lhs, rhs, cache } => f
                .debug_struct("Div")
                .field("lhs", lhs)
                .field("rhs", rhs)
                .field("cache", cache)
                .finish(),
            Self::Sin { arg, cache } => f
                .debug_struct("Sin")
                .field("arg", arg)
                .field("cache", cache)
                .finish(),
            Self::Cos { arg, cache } => f
                .debug_struct("Cos")
                .field("arg", arg)
                .field("cache", cache)
                .finish(),
            Self::Pow { arg, power, cache } => f
                .debug_struct("Pow")
                .field("arg", arg)
                .field("power", power)
                .field("cache", cache)
                .finish(),
        }
    }
}

pub fn create_input(name: &'static str) -> CompNode {
    Rc::new(BoxedCompNode::Input {
        name,
        value: Rc::new(Cell::new(None)),
        deps: Rc::new(RefCell::new(Vec::new())),
    })
}

pub fn create_input_from(name: &'static str, value: f32) -> CompNode {
    Rc::new(BoxedCompNode::Input {
        name,
        value: Rc::new(Cell::new(Some(value))),
        deps: Rc::new(RefCell::new(Vec::new())),
    })
}

pub fn sum(lhs: CompNode, rhs: CompNode) -> CompNode {
    let result = Rc::new(BoxedCompNode::Sum {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
        cache: Cell::new(None),
    });

    lhs.update_deps(Rc::downgrade(&result));
    rhs.update_deps(Rc::downgrade(&result));

    result
}

pub fn mul(lhs: CompNode, rhs: CompNode) -> CompNode {
    let result = Rc::new(BoxedCompNode::Mul {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
        cache: Cell::new(None),
    });

    lhs.update_deps(Rc::downgrade(&result));
    rhs.update_deps(Rc::downgrade(&result));

    result
}

pub fn sub(lhs: CompNode, rhs: CompNode) -> CompNode {
    let result = Rc::new(BoxedCompNode::Sub {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
        cache: Cell::new(None),
    });

    lhs.update_deps(Rc::downgrade(&result));
    rhs.update_deps(Rc::downgrade(&result));

    result
}

pub fn div(lhs: CompNode, rhs: CompNode) -> CompNode {
    let result = Rc::new(BoxedCompNode::Div {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
        cache: Cell::new(None),
    });

    lhs.update_deps(Rc::downgrade(&result));
    rhs.update_deps(Rc::downgrade(&result));

    result
}

pub fn sin(arg: CompNode) -> CompNode {
    let result = Rc::new(BoxedCompNode::Sin {
        arg: arg.clone(),
        cache: Cell::new(None),
    });

    arg.update_deps(Rc::downgrade(&result));

    result
}

pub fn cos(arg: CompNode) -> CompNode {
    let result = Rc::new(BoxedCompNode::Cos {
        arg: arg.clone(),
        cache: Cell::new(None),
    });

    arg.update_deps(Rc::downgrade(&result));

    result
}

pub fn pow(arg: CompNode, power: CompNode) -> CompNode {
    let result = Rc::new(BoxedCompNode::Pow {
        arg: arg.clone(),
        power: power.clone(),
        cache: Cell::new(None),
    });

    arg.update_deps(Rc::downgrade(&result));
    power.update_deps(Rc::downgrade(&result));

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
        let power = create_input_from("pw", gen_b as f32);

        let expr = pow(arg, power);

        assert_eq!(f32::powf(gen_a as f32, gen_b as f32), expr.compute())
    }

    // TODO: add compound tests sum(mul()), div(sum) etc
}
