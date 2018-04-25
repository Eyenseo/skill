use common::internal::SkillObject;

pub struct UndefinedObject {}

impl UndefinedObject {
    pub fn new() -> UndefinedObject {
        UndefinedObject {}
    }
}
impl SkillObject for UndefinedObject {}
