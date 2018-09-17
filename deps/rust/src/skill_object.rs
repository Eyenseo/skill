/*
 * @author Roland Jaeger
 */

use common::error::*;

pub(crate) const DELETE: usize = std::usize::MAX;

// NOTE This is a shotgun to the foot, this has to be crate local - congratulations rust
pub trait Deletable {
    fn mark_for_deletion(&mut self);
    fn to_delete(&self) -> bool;
}

pub trait SkillObject: Deletable {
    fn skill_type_id(&self) -> usize;

    fn get_skill_id(&self) -> usize;
    // NOTE this should be mutable but is not because of String - have a look at SkillString
    // NOTE this has to check that id != DELETE
    fn set_skill_id(&self, id: usize) -> Result<(), SkillFail>;
}
