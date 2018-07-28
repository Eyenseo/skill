use common::internal::InstancePool;
use common::internal::SkillObject;
use common::iterator::type_hierarchy;
use common::Ptr;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Iter {
    type_hierarchy: type_hierarchy::Iter,
    pool: Rc<RefCell<InstancePool>>,
    instance_index: usize,
    instance_end: usize,
    block_index: usize,
    block_end: usize,
}

impl Iter {
    pub fn new(pool: Rc<RefCell<InstancePool>>) -> Iter {
        let mut iter = Iter {
            block_end: pool.borrow().blocks().len(),
            type_hierarchy: type_hierarchy::Iter::new(pool.clone()),
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
            let block = &pool.blocks()[self.block_index];
            self.instance_index = block.bpo;
            self.instance_end = self.instance_index + block.static_count;
            self.block_index += 1;
        }
        // If no new instance is available iterate over the new instances that were added
        if self.instance_index == self.instance_end && self.block_index == self.block_end {
            self.block_index += 1;
            self.instance_index = 0;
            self.instance_end = self.pool.borrow().new_instances().len();
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
            let ret = self.pool.borrow().static_instances()[self.instance_index].clone();
            self.instance_index += 1; // TODO NOT a pointer?! SkillID - starts form 1

            if self.instance_index == self.instance_end {
                self.next_viable();
            }
            Some(ret)
        } else {
            let ret = self.pool.borrow().new_instances()[self.instance_index].clone();
            self.instance_index += 1; // Array position - starts from 0
            Some(ret)
        }
        // FIXME where is the NONE?
    }
}
