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

    fn set_skill_id(&self, id: usize) {
        self.id.set(id);
    }
}
