use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

#[derive(Clone, Debug)]
enum BoxedCompNode {
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

impl BoxedCompNode {
    fn update_deps(&self, dep: Weak<Self>) {
        match self {
            Self::Input {
                name: _,
                value: _,
                deps,
            } => deps.borrow_mut().push(dep),
            Self::Sum { lhs, rhs, cache: _ } | Self::Mul { lhs, rhs, cache: _ } => {
                lhs.update_deps(dep.clone());
                rhs.update_deps(dep);
            }
        }
    }

    fn as_string(&self) -> String {
        match self {
            Self::Input {
                name,
                value,
                deps: _,
            } => format!("{}({:?})", name, value),
            Self::Sum { lhs, rhs, cache: _ } => {
                format!("({} + {})", lhs.as_string(), rhs.as_string())
            }
            Self::Mul { lhs, rhs, cache: _ } => {
                format!("({} * {})", lhs.as_string(), rhs.as_string())
            }
        }
    }

    fn set(&self, val: f32) {
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
            } => cache.set(None),
            _ => (),
        }
    }

    fn compute(&self) -> f32 {
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
                    let rslt = lhs.compute() + rhs.compute();
                    cache.set(Some(rslt));
                    rslt
                }
            }
            Self::Mul { lhs, rhs, cache } => {
                if let Some(cached_value) = cache.get() {
                    cached_value
                } else {
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

fn create_input(name: &'static str) -> BoxedCompNode {
    BoxedCompNode::Input {
        name,
        value: Rc::new(Cell::new(None)),
        deps: Rc::new(RefCell::new(Vec::new())),
    }
}

fn sum(lhs: BoxedCompNode, rhs: BoxedCompNode) -> BoxedCompNode {
    let lhs = Rc::new(lhs);
    let rhs = Rc::new(rhs);

    lhs.update_deps(Rc::downgrade(&lhs));
    rhs.update_deps(Rc::downgrade(&rhs));

    BoxedCompNode::Sum {
        lhs,
        rhs,
        cache: Cell::new(None),
    }
}

fn mul(lhs: BoxedCompNode, rhs: BoxedCompNode) -> BoxedCompNode {
    let lhs = Rc::new(lhs);
    let rhs = Rc::new(rhs);

    lhs.update_deps(Rc::downgrade(&lhs));
    rhs.update_deps(Rc::downgrade(&rhs));

    BoxedCompNode::Mul {
        lhs,
        rhs,
        cache: Cell::new(None),
    }
}

fn main() {
    let a = create_input("a");
    let b = create_input("b");
    let c = create_input("c");

    let rslt = sum(a.clone(), mul(b.clone(), c.clone()));

    println!("{}", rslt.as_string());

    a.set(10.);
    b.set(50.);
    c.set(30.);

    println!("{}", rslt.as_string());

    println!("compute : {}", rslt.compute());
}
