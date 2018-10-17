use common::internal::*;
use common::*;

use std::cell::RefCell;
use std::rc::Rc;

/// Iterator that iterates over all instances of a single type
#[derive(Clone)]
pub struct Iter {
    pool: Rc<RefCell<PoolProxy>>,
    instance_index: usize,
    instance_end: usize,
    block_index: usize,
    block_end: usize,
}

impl Iter {
    pub(crate) fn new(pool: Rc<RefCell<PoolProxy>>) -> Iter {
        let mut iter = Iter {
            block_end: pool.borrow().pool().blocks().len(),
            pool: pool.clone(),
            instance_index: 0,
            block_index: 0,
            instance_end: 0,
        };
        iter.next_viable();
        iter
    }

    fn next_viable(&mut self) {
        // Get the next instance from the pool
        loop {
            if self.instance_index != self.instance_end || self.block_index >= self.block_end {
                break;
            }
            let pool = self.pool.borrow();
            let pool = pool.pool();
            let block = &pool.blocks()[self.block_index];
            self.instance_index = block.bpo;
            self.instance_end = self.instance_index + block.static_count;
            self.block_index += 1;
        }
        // If no new instance is available iterate over the new instances that were added
        if self.instance_index == self.instance_end && self.block_index == self.block_end {
            self.instance_index = 0;
            self.instance_end = self.pool.borrow().pool().new_instances().len();
            self.block_index += 1;
        }
    }
}

impl Iterator for Iter {
    type Item = Ptr<SkillObject>;

    /// Iterates over all deserialized instances of a single InstancePool and continues then with
    /// the newly created instances of the InstancePool's type.
    fn next(&mut self) -> Option<Ptr<SkillObject>> {
        if self.instance_index >= self.instance_end {
            return None;
        }

        if self.block_index <= self.block_end {
            let tmp = self.pool.borrow().pool().get_base_vec();
            let ret = tmp.borrow()[self.instance_index].clone();
            self.instance_index += 1;

            if self.instance_index == self.instance_end {
                self.next_viable();
            }
            Some(ret)
        } else {
            let ret = self.pool.borrow().pool().new_instances()[self.instance_index].clone();
            self.instance_index += 1;
            Some(ret)
        }
    }
}
