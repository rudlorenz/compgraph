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
}

pub type CompNode = Rc<BoxedCompNode>;

impl BoxedCompNode {
    fn update_deps(&self, dep: Weak<Self>) {
        match self {
            Self::Input {
                name: _,
                value: _,
                deps,
            } => deps.borrow_mut().push(dep),
            Self::Sum { lhs, rhs, .. } | Self::Mul { lhs, rhs, .. } => {
                lhs.update_deps(dep.clone());
                rhs.update_deps(dep);
            }
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
            Self::Sum {
                lhs: _,
                rhs: _,
                cache,
            }
            | Self::Mul {
                lhs: _,
                rhs: _,
                cache,
            } => {
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
                    let rslt = lhs.compute() + rhs.compute();
                    cache.set(Some(rslt));
                    rslt
                }
            }
            Self::Mul { lhs, rhs, cache } => {
                if let Some(cached_value) = cache.get() {
                    cached_value
                } else {
                    log::debug!("Cache miss {}", self);
                    let rslt = lhs.compute() * rhs.compute();
                    cache.set(Some(rslt));
                    rslt
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

#[cfg(test)]
mod tests {
    use crate::{create_input_from, mul, sum};

    #[test]
    fn test_sum() {
        let a = create_input_from("a", 2.);
        let b = create_input_from("b", 2.);
        let expr = sum(a, b);

        assert_eq!(4., expr.compute())
    }

    #[test]
    fn test_mul() {
        let a = create_input_from("a", 2.);
        let b = create_input_from("b", 2.);
        let expr = mul(a, b);

        assert_eq!(4., expr.compute())
    }

    #[test]
    fn compund_mul() {
        let a = create_input_from("a", 2.);
        let b = create_input_from("b", 3.);
        let c = create_input_from("c", 4.);

        // a * (b * c)
        let expr = mul(a, mul(b, c));

        assert_eq!(24., expr.compute())
    }

    #[test]
    fn compound_sum() {
        let a = create_input_from("a", 2.);
        let b = create_input_from("b", 3.);
        let c = create_input_from("c", 4.);

        // a + (b + c)
        let expr = sum(a, sum(b, c));

        assert_eq!(9., expr.compute())
    }

    #[test]
    fn compund_mul_sum() {
        let a = create_input_from("a", 2.);
        let b = create_input_from("b", 3.);
        let c = create_input_from("c", 4.);

        // a + (b * c)
        let expr = sum(a, mul(b, c));

        assert_eq!(14., expr.compute())
    }

    #[test]
    fn compound_sum_mul() {
        let a = create_input_from("a", 5.);
        let b = create_input_from("b", 3.);
        let c = create_input_from("c", 4.);

        // a * (b + c)
        let expr = mul(a, sum(b, c));

        assert_eq!(35., expr.compute())
    }
}
