/*
 * @author Roland Jaeger
 */

use common::error::*;

pub(crate) const DELETE: usize = std::usize::MAX;

/// This is a shotgun to the foot, this has to be crate local but that is not possible in Rust
/// - congratulations
pub trait Deletable {
    fn mark_for_deletion(&mut self);
    fn to_delete(&self) -> bool;
}

/// Base trait for all UserTypes / types that are used in Pool
pub trait SkillObject: Deletable {
    fn skill_type_id(&self) -> usize;

    fn get_skill_id(&self) -> usize;
    // NOTE this should be mutable but is not because of String - have a look at SkillString
    // NOTE this has to check that id != DELETE
    fn set_skill_id(&self, id: usize) -> Result<(), SkillFail>;
}
