use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

#[derive(Clone)]
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

type CompNode = Rc<BoxedCompNode>;

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
            } => {
                cache.set(None)
            }
            Self::Input { .. } => (),
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

fn create_input(name: &'static str) -> CompNode {
    Rc::new(BoxedCompNode::Input {
        name,
        value: Rc::new(Cell::new(None)),
        deps: Rc::new(RefCell::new(Vec::new())),
    })
}

fn sum(lhs: CompNode, rhs: CompNode) -> CompNode {
    let result = Rc::new(BoxedCompNode::Sum {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
        cache: Cell::new(None),
    });

    lhs.update_deps(Rc::downgrade(&result));
    rhs.update_deps(Rc::downgrade(&result));

    result
}

fn mul(lhs: CompNode, rhs: CompNode) -> CompNode {
    let result = Rc::new(BoxedCompNode::Mul {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
        cache: Cell::new(None),
    });

    lhs.update_deps(Rc::downgrade(&result));
    rhs.update_deps(Rc::downgrade(&result));

    result
}

fn main() {
    let a = create_input("a");
    let b = create_input("b");
    let c = create_input("c");

    let rslt = sum(a.clone(), mul(b.clone(), c.clone()));

    println!("{rslt}");

    a.set(10.);
    b.set(50.);
    c.set(30.);

    println!("{rslt}");

    println!("compute : {}", rslt.compute());
}
