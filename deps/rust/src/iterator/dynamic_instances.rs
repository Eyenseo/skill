/*
 * @author Roland Jaeger
 */

use common::error::*;
use common::internal::*;
use common::iterator::type_hierarchy;
use common::*;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct Iter {
    type_hierarchy: type_hierarchy::Iter,
    current: Option<Rc<RefCell<PoolProxy>>>,
    instance_index: usize,
    instance_end: usize,
    block_index: usize,
    block_end: usize,
}

impl Iter {
    pub(crate) fn new(pool: Rc<RefCell<PoolProxy>>) -> Result<Iter, SkillFail> {
        let mut iter = Iter {
            block_end: pool.borrow().pool().blocks().len(),
            type_hierarchy: type_hierarchy::Iter::new(pool.clone())?,
            current: Some(pool.clone()),
            instance_index: 0,
            block_index: 0,
            instance_end: 0,
        };
        iter.next_viable();
        Ok(iter)
    }

    fn next_viable(&mut self) {
        // Get the next instance from the base pool
        {
            let pool = self.current.as_ref().unwrap().borrow(); // this has to be checked in the calling methods
            let pool = pool.pool();
            loop {
                if self.instance_index != self.instance_end || self.block_index >= self.block_end {
                    break;
                }
                let block = &pool.blocks()[self.block_index];
                self.instance_index = block.bpo;
                self.instance_end = self.instance_index + block.dynamic_count;
                self.block_index += 1;
            }
        }
        // If no further instance is available go trough the type hierarchy and their new instances
        if self.instance_index == self.instance_end && self.block_index == self.block_end {
            self.block_index += 1;
            loop {
                if let Some(p) = self.type_hierarchy.next() {
                    let objs = p.borrow().pool().new_instances().len();
                    if objs > 0 {
                        self.instance_index = 0;
                        self.instance_end = objs;
                        self.current = Some(p);
                        return; // return else we assign None
                    }
                } else {
                    break;
                }
            }
            self.current = None;
        }
    }
}

impl Iterator for Iter {
    type Item = Ptr<SkillObject>;

    /// Iterates over all deserialized instances of the type hierarchy and continues then with the
    /// newly created instances of the type hierarchy.
    fn next(&mut self) -> Option<Ptr<SkillObject>> {
        if self.instance_index >= self.instance_end {
            return None;
        }

        if let Some(pool) = self.current.clone()
        // clone because borrow madness
        {
            if self.block_index <= self.block_end {
                let tmp = pool.borrow().pool().get_base_vec();
                let ret = tmp.borrow()[self.instance_index].clone();
                self.instance_index += 1;

                self.next_viable();

                Some(ret)
            } else {
                let ret = pool.borrow().pool().new_instances()[self.instance_index].clone();
                self.instance_index += 1;

                if self.instance_index == self.instance_end {
                    loop {
                        if let Some(p) = self.type_hierarchy.next() {
                            let objs = p.borrow().pool().new_instances().len();
                            if objs > 0 {
                                self.instance_index = 0;
                                self.instance_end = objs;
                                self.current = Some(p);
                                return Some(ret);
                            }
                        } else {
                            break;
                        }
                    }
                }
                self.current = None;
                None
            }
        } else {
            None
        }
    }
}