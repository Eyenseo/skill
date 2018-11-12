use common::error::*;
use common::internal::io::*;
use common::internal::*;
use common::*;
use SkillFile;
use SkillFileBuilder;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Used to provide [`Pool`]'s special functions. In this case to make
/// [`foreign::FieldDeclaration`] instances and [`foreign::Foreign`] instances
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
            Box::new(RefCell::new(FieldDeclaration::new(Box::new(
                foreign::FieldIO::new(field_name, index, field_type),
            )))),
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

/// Manages all [foreign][`foreign::Foreign`] instances.
pub struct Pool {
    pool: internal::Pool,
}

impl Pool {
    pub(crate) fn new(
        string_block: Rc<RefCell<StringBlock>>,
        name: Rc<SkillString>,
        type_id: usize,
        super_pool: Option<Rc<RefCell<PoolProxy>>>,
    ) -> Pool {
        Pool {
            pool: internal::Pool::new(
                string_block,
                name,
                type_id,
                Box::new(Maker::new(super_pool)),
            ),
        }
    }

    /// Used to explicitly deserialize foreign fields or obtain field information
    ///
    /// # Returns
    /// All field declaration that this type has
    pub fn fields(&self) -> &Vec<Box<RefCell<FieldDeclaration>>> {
        self.pool.fields()
    }

    /// Used to initialize a specific field of a type hierarchy
    ///
    /// # Arguments
    /// * `name` - Name of the field to initialize
    pub fn initialize_field(&self, name: &str) -> Result<(), SkillFail> {
        self.pool.initialize_field(&name)
    }

    /// Used to initialize all fields of a type hierarchy
    pub fn initialize_all(&self) -> Result<(), SkillFail> {
        self.pool.initialize_all_fields()
    }

    /// # Returns
    /// Name of this type
    pub fn name(&self) -> &Rc<SkillString> {
        self.pool.name()
    }

    /// # Arguments
    /// * `index` - Index/ID of the instance to obtain
    ///
    /// # Returns
    /// User type instance of given id/index
    pub fn get(&self, index: usize) -> Result<Ptr<foreign::Foreign>, SkillFail> {
        match self.pool.read_object(index) {
            Ok(obj) => {
                if obj.borrow().get_skill_id() == skill_object::DELETE {
                    return Err(SkillFail::user(UserFail::AccessDeleted));
                }
                match obj.cast::<foreign::Foreign>() {
                    Some(obj) => Ok(obj.clone()),
                    None => Err(SkillFail::user(UserFail::BadCastID { id: index })),
                }
            }
            Err(e) => Err(e),
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
