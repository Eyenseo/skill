use common::error::*;
use common::internal::InstancePool;
use common::SkillString;

use std::cell::RefCell;
use std::rc::Rc;

pub trait PoolMaker {
    fn make_pool(
        &mut self,
        type_name: &Rc<SkillString>,
        type_id: usize,
        super_pool: Option<Rc<RefCell<InstancePool>>>,
    ) -> Result<Rc<RefCell<InstancePool>>, SkillFail>;

    fn get_pool(&self, type_name_index: usize) -> Option<Rc<RefCell<InstancePool>>>;
}
