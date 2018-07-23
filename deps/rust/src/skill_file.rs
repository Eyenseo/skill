use common::internal::InstancePool;
use common::SkillError;

use std::cell::RefCell;
use std::rc::Rc;

pub trait SkillFile {
    type T;

    fn open(file: &str) -> Result<Self::T, SkillError>;
    fn create(file: &str) -> Result<Self::T, SkillError>;
    fn write(&self) -> Result<(), SkillError>;
    fn close(self) -> Result<(), SkillError>;
    fn check(&self) -> Result<(), SkillError>;
}

pub trait PoolMaker {
    fn make_pool(
        &mut self,
        type_name_index: usize,
        type_name: &str,
        type_id: usize,
        super_pool: Option<Rc<RefCell<InstancePool>>>,
    ) -> Rc<RefCell<InstancePool>>;

    fn get_pool(&self, type_name_index: usize) -> Option<Rc<RefCell<InstancePool>>>;
}
