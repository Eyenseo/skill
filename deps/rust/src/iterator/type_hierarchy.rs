use common::internal::InstancePool;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct Iter {
    current: Option<Rc<RefCell<InstancePool>>>,
    type_hierarchy_height: usize,
}

impl Iter {
    /// * `pool` has to be the base pool of a type heirarchy
    pub fn new(pool: Rc<RefCell<InstancePool>>) -> Iter {
        if !pool.borrow().is_base() {
            panic!("The pool has to be a base pool");
        }
        Iter {
            type_hierarchy_height: pool.borrow().type_hierarchy_height(),
            current: Some(pool.clone()),
        }
    }
}

impl Iterator for Iter {
    type Item = Rc<RefCell<InstancePool>>;

    /// Iterates over all pools in the type hierarchy
    fn next(&mut self) -> Option<Rc<RefCell<InstancePool>>> {
        let ret = self.current.clone();

        if let Some(current) = self.current.clone()
        // clone because of borrow madness
        {
            let next = current.borrow().get_next_pool();

            if let Some(next) = next {
                if next.borrow().type_hierarchy_height() > self.type_hierarchy_height {
                    self.current = Some(next);
                } else {
                    self.current = None;
                }
            } else {
                self.current = None;
            }
        }

        ret
    }
}
