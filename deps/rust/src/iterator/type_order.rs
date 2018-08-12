use common::error::*;
use common::internal::*;
use common::iterator::*;
use common::*;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub(crate) struct Iter {
    type_hierarchy: type_hierarchy::Iter,
    static_data: Option<static_data::Iter>,
}

impl Iter {
    pub(crate) fn new(pool: Rc<RefCell<PoolProxy>>) -> Result<Iter, SkillFail> {
        let mut iter = Iter {
            type_hierarchy: type_hierarchy::Iter::new(pool.clone())?,
            static_data: None,
        };
        iter.next_viable();
        Ok(iter)
    }

    fn next_viable(&mut self) {
        loop {
            if let Some(p) = self.type_hierarchy.next() {
                if p.borrow().pool().static_size() > 0 {
                    self.static_data = Some(static_data::Iter::new(p));
                    return; // return else we assign None
                }
            } else {
                break;
            }
        }
        self.static_data = None;
    }
}

impl Iterator for Iter {
    type Item = Ptr<SkillObject>;

    /// Iterates over all deserialized instances of a type followed by the new instances of the
    /// same type before advancing to the next pool
    fn next(&mut self) -> Option<Ptr<SkillObject>> {
        if self.static_data.is_none() {
            return None;
        }

        if let Some(ret) = self.static_data.as_mut().unwrap().next() {
            Some(ret)
        } else {
            self.next_viable();
            self.next()
        }
    }
}
