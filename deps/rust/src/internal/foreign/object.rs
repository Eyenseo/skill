/*
 * @author Roland Jaeger
 */

use common::error::*;
use common::internal::*;
use common::*;

use std::cell::Cell;

/// Struct that is used to represent instances of types that where not known
/// at generation time and that do not have a known super type
#[derive(Default, Debug)]
#[repr(C)]
pub(crate) struct Foreign {
    // NOTE be sure to change PoolMaker::genTypeStruct too!
    skill_id: Cell<usize>,
    skill_type_id: usize,
    foreign_data: Vec<foreign::FieldData>,
}
/// Accessor trait
pub(crate) trait ForeignObject: SkillObject {
    fn foreign_fields(&self) -> &Vec<foreign::FieldData>;
    fn foreign_fields_mut(&mut self) -> &mut Vec<foreign::FieldData>;
}

impl Foreign {
    pub(crate) fn new(skill_id: usize, skill_type_id: usize) -> Foreign {
        Foreign {
            skill_id: Cell::new(skill_id),
            skill_type_id,
            foreign_data: Vec::default(),
        }
    }
}

impl ForeignObject for Foreign {
    fn foreign_fields(&self) -> &Vec<foreign::FieldData> {
        &self.foreign_data
    }
    fn foreign_fields_mut(&mut self) -> &mut Vec<foreign::FieldData> {
        &mut self.foreign_data
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
