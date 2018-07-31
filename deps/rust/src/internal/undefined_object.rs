use common::error::*;
use common::internal::skill_object;
use common::internal::SkillObject;

use std::cell::Cell;

#[derive(Default, Debug)]
pub struct UndefinedObject {
    id: Cell<usize>,
}

impl UndefinedObject {
    pub fn new(id: usize) -> UndefinedObject {
        UndefinedObject { id: Cell::new(id) }
    }
}

impl SkillObject for UndefinedObject {
    fn get_skill_id(&self) -> usize {
        self.id.get()
    }

    fn set_skill_id(&self, id: usize) -> Result<(), SkillFail> {
        if id == skill_object::DELETE {
            return Err(SkillFail::internal(InternalFail::ReservedID { id }));
        }
        self.id.set(id);
        Ok(())
    }

    fn mark_for_pruning(&self) {
        self.id.set(skill_object::DELETE);
    }
    fn to_prune(&self) -> bool {
        self.id.get() == skill_object::DELETE
    }
}
