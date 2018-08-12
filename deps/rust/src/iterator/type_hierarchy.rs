use common::error::*;
use common::internal::*;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, Clone)]
pub(crate) struct Iter {
    current: Option<Rc<RefCell<PoolProxy>>>,
    type_hierarchy_height: usize,
}

impl Iter {
    pub(crate) fn new(pool: Rc<RefCell<PoolProxy>>) -> Result<Iter, SkillFail> {
        Ok(Iter {
            type_hierarchy_height: pool.borrow().pool().type_hierarchy_height(),
            current: Some(pool.clone()),
        })
    }
}

impl Iterator for Iter {
    type Item = Rc<RefCell<PoolProxy>>;

    /// Iterates over all pools in the type hierarchy
    fn next(&mut self) -> Option<Rc<RefCell<PoolProxy>>> {
        let ret = self.current.clone();

        if let Some(current) = self.current.clone()
        // clone because of borrow madness
        {
            let next = current.borrow().pool().get_next_pool();

            if let Some(next) = next {
                if next.borrow().pool().type_hierarchy_height() > self.type_hierarchy_height {
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
