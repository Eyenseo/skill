use common::error::*;
use common::internal::foreign;
use common::internal::skill_object;
use common::internal::SkillObject;

use std::cell::Cell;

#[derive(Default, Debug)]
pub(crate) struct ObjectProper {
    skill_id: Cell<usize>,
    skill_type_id: usize,
    undefind_data: Vec<foreign::FieldData>,
}

pub(crate) trait Object: SkillObject {
    fn foreign_fields(&self) -> &Vec<foreign::FieldData>;
    fn foreign_fields_mut(&mut self) -> &mut Vec<foreign::FieldData>;
}

impl ObjectProper {
    pub fn new(skill_id: usize, skill_type_id: usize) -> ObjectProper {
        ObjectProper {
            skill_id: Cell::new(skill_id),
            skill_type_id,
            undefind_data: Vec::default(),
        }
    }
}

impl Object for ObjectProper {
    fn foreign_fields(&self) -> &Vec<foreign::FieldData> {
        &self.undefind_data
    }
    fn foreign_fields_mut(&mut self) -> &mut Vec<foreign::FieldData> {
        &mut self.undefind_data
    }
}

impl SkillObject for ObjectProper {
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

    fn mark_for_pruning(&self) {
        self.skill_id.set(skill_object::DELETE);
    }
    fn to_prune(&self) -> bool {
        self.skill_id.get() == skill_object::DELETE
    }
}
