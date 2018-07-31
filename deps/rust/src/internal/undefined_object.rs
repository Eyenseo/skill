use common::error::*;
use common::internal::skill_object;
use common::internal::SkillObject;

use std::cell::Cell;

#[derive(Default, Debug)]
pub struct UndefinedObject {
    skill_id: Cell<usize>,
    skill_type_id: usize,
}

impl UndefinedObject {
    pub fn new(skill_id: usize, skill_type_id: usize) -> UndefinedObject {
        UndefinedObject {
            skill_id: Cell::new(skill_id),
            skill_type_id,
        }
    }
}

impl SkillObject for UndefinedObject {
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
