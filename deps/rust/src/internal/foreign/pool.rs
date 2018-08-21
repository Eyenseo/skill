use common::error::*;
use common::internal::io::*;
use common::internal::*;
use common::*;
use SkillFile;
use SkillFileBuilder;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Maker {
    super_pool: Option<Weak<RefCell<PoolProxy>>>,
}

impl Maker {
    pub(crate) fn new(super_pool: Option<Rc<RefCell<PoolProxy>>>) -> Maker {
        Maker {
            super_pool: if let Some(pool) = super_pool {
                Some(Rc::downgrade(&pool))
            } else {
                None
            },
        }
    }
}

impl PoolPartsMaker for Maker {
    fn make_field(
        &self,
        index: usize,
        field_name: Rc<SkillString>,
        field_type: FieldType,
        string_pool: &StringBlock,
    ) -> Result<(bool, Box<RefCell<FieldDeclaration>>), SkillFail> {
        Ok((
            true,
            Box::new(RefCell::new(foreign::FieldDeclaration::new(
                field_name, index, field_type,
            ))),
        ))
    }

    fn make_instance(&self, skill_id: usize, skill_type_id: usize) -> Ptr<SkillObject> {
        if let Some(pool) = self.super_pool.as_ref() {
            return pool
                .upgrade()
                .unwrap()
                .borrow()
                .pool()
                .make_foreign(skill_id, skill_type_id);
        }
        trace!(
            target: "SkillParsing",
            "Create new ObjectProper",
        );
        Ptr::new(foreign::ObjectProper::new(skill_id, skill_type_id))
    }

    fn make_foreign(&self, skill_id: usize, skill_type_id: usize) -> Ptr<SkillObject> {
        if let Some(pool) = self.super_pool.as_ref() {
            return pool
                .upgrade()
                .unwrap()
                .borrow()
                .pool()
                .make_foreign(skill_id, skill_type_id);
        }
        trace!(
            target: "SkillParsing",
            "Create new ObjectProper",
        );
        Ptr::new(foreign::ObjectProper::new(skill_id, skill_type_id))
    }
}

pub(crate) struct Pool {
    pool: internal::Pool,
}

impl Pool {
    pub(crate) fn new(
        name: Rc<SkillString>,
        type_id: usize,
        super_pool: Option<Rc<RefCell<PoolProxy>>>,
    ) -> Pool {
        Pool {
            pool: internal::Pool::new(name, type_id, Box::new(Maker::new(super_pool))),
        }
    }
}

impl PoolProxy for Pool {
    fn pool(&self) -> &internal::Pool {
        &self.pool
    }
    fn pool_mut(&mut self) -> &mut internal::Pool {
        &mut self.pool
    }

    fn complete(&mut self, file: &SkillFileBuilder) {}
}
