use common::error::*;
use common::internal::InstancePool;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct Iter {
    current: Option<Rc<RefCell<InstancePool>>>,
    type_hierarchy_height: usize,
}

impl Iter {
    pub fn new(pool: Rc<RefCell<InstancePool>>) -> Result<Iter, SkillFail> {
        Ok(Iter {
            type_hierarchy_height: pool.borrow().type_hierarchy_height(),
            current: Some(pool.clone()),
        })
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
