pub trait SkillObject {
    fn get_skill_id(&self) -> usize;
    // NOTE this should be mutable but is not because of String - have a look at SkillString
    fn set_skill_id(&self, id: usize);
}
