use common::internal::SkillObject;

#[derive(Default, Copy, Clone, Debug)]
pub struct UndefinedObject {}

impl UndefinedObject {
    pub fn new() -> UndefinedObject {
        UndefinedObject {}
    }
}
impl SkillObject for UndefinedObject {}
