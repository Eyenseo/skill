use common::error::*;
use common::internal::*;
use common::*;

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

/// Struct that is used to represent instances of types that where not known
/// at generation time and that do not have a known super type.
///
/// Its main use is to implement [`foreign::ForeignObject`] that allows to access [field data][`foreign::FieldData`] of fields that were not known at generation time.
#[derive(Default, Debug)]
#[repr(C)]
pub struct Foreign {
    // NOTE be sure to change PoolMaker::genTypeStruct too!
    skill_id: Cell<usize>,
    skill_type_id: usize,
    foreign_fields: HashMap<Rc<SkillString>, foreign::FieldData>,
}

/// Accessor trait for all fields that where not known at generation time.
///
/// Users can use this trait's functions to obtain [field data][`foreign::FieldData`] of these
/// foreign fields to work with them.
pub trait ForeignObject: SkillObject {
    /// # Returns
    /// Map of field identifiers to immutable [field data][`foreign::FieldData`]
    fn foreign_fields(&self) -> &HashMap<Rc<SkillString>, foreign::FieldData>;
    /// # Returns
    /// Map of field identifiers to mutable [field data][`foreign::FieldData`]
    fn foreign_fields_mut(&mut self) -> &mut HashMap<Rc<SkillString>, foreign::FieldData>;
}

impl Foreign {
    pub(crate) fn new(skill_id: usize, skill_type_id: usize) -> Foreign {
        Foreign {
            skill_id: Cell::new(skill_id),
            skill_type_id,
            foreign_fields: HashMap::default(),
        }
    }
}

impl ForeignObject for Foreign {
    fn foreign_fields(&self) -> &HashMap<Rc<SkillString>, foreign::FieldData> {
        &self.foreign_fields
    }
    fn foreign_fields_mut(&mut self) -> &mut HashMap<Rc<SkillString>, foreign::FieldData> {
        &mut self.foreign_fields
    }
}

impl SkillObject for Foreign {
    fn skill_type_id(&self) -> usize {
        self.skill_type_id
    }
    fn get_skill_id(&self) -> usize {
        self.skill_id.get()
    }

    fn set_skill_id(&self, skill_id: usize) -> Result<(), SkillFail> {
        if skill_id == skill_object::DELETE {
            return Err(SkillFail::internal(InternalFail::ReservedID {
                id: skill_id,
            }));
        }
        self.skill_id.set(skill_id);
        Ok(())
    }
}

impl Deletable for Foreign {
    fn mark_for_deletion(&mut self) {
        self.skill_id.set(skill_object::DELETE);
    }
    fn to_delete(&self) -> bool {
        self.skill_id.get() == skill_object::DELETE
    }
}
