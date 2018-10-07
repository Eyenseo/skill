/*
 * @author Roland Jaeger
 */

use common::error::*;
use common::internal::io::*;
use common::internal::*;
use common::*;
use SkillFile;
use SkillFileBuilder;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Struct that is used to provide Pool special functions. In this case to
/// make foreign::FieldDeclaration instances and Foreign instances
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
        // we know nothing at generation time so everything must be a foreign
        // FieldDeclaration
        Ok((
            true,
            Box::new(RefCell::new(foreign::FieldDeclaration::new(
                field_name, index, field_type,
            ))),
        ))
    }

    fn make_instance(&self, skill_id: usize, skill_type_id: usize) -> Ptr<SkillObject> {
        // In case there is a super pool use that to create the instances
        // In this case it is not strictly necessary but kept for homogeneous
        // behaviour
        if let Some(pool) = self.super_pool.as_ref() {
            return pool
                .upgrade()
                .unwrap()
                .borrow()
                .pool()
                .make_instance(skill_id, skill_type_id);
        }
        trace!(
            target: "SkillParsing",
            "Create new ObjectProper",
        );
        Ptr::new(foreign::Foreign::new(skill_id, skill_type_id))
    }
}

/// struct that manages all Foreign instances
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
