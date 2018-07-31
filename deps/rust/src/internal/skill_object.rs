use common::error::*;

pub const DELETE: usize = std::usize::MAX;

pub trait SkillObject {
    fn skill_type_id(&self) -> usize;

    fn get_skill_id(&self) -> usize;
    // NOTE this should be mutable but is not because of String - have a look at SkillString
    // NOTE this has to chekc that id != DELETE
    fn set_skill_id(&self, id: usize) -> Result<(), SkillFail>;

    fn mark_for_pruning(&self);
    fn to_prune(&self) -> bool;
}
