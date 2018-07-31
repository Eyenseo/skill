use common::error::*;
use common::internal::skill_object;
use common::internal::SkillObject;

use std::cell::Cell;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Default)]
pub struct SkillString {
    // NOTE this is bad! but since we want to modify id without having to rely on RefCell (we need
    // to be hashable) this is our only option
    id: Cell<usize>,
    string: String,
    hash: u64,
}

impl SkillString {
    pub fn new(id: usize, string: &str) -> SkillString {
        SkillString {
            id: Cell::new(id),
            string: String::from(string),
            hash: SkillString::gen_hash(string),
        }
    }

    pub fn string(&self) -> &String {
        &self.string
    }
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn gen_hash(string: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        string.hash(&mut hasher);
        hasher.finish()
    }
}

impl SkillObject for SkillString {
    fn get_skill_id(&self) -> usize {
        self.id.get()
    }
    fn set_skill_id(&self, id: usize) -> Result<(), SkillFail> {
        if id == skill_object::DELETE {
            return Err(SkillFail::user(UserFail::ReservedID { id }));
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

impl From<std::borrow::Cow<'static, str>> for SkillString {
    fn from(string: std::borrow::Cow<'static, str>) -> Self {
        SkillString {
            id: Cell::new(0), // TODO is 0 ok or is it reserved for something else?
            hash: SkillString::gen_hash(string.as_ref()),
            string: String::from(string),
        }
    }
}

impl Hash for SkillString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // NOTE I guess this means that this will be rehashed? ... so much to "blazingly fast"
        state.write_u64(self.hash);
    }
}

impl fmt::Display for SkillString {
    default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.string.fmt(f)?;
        Ok(())
    }
}

impl PartialEq for SkillString {
    fn eq(&self, other: &SkillString) -> bool {
        self.hash == other.hash
    }
}

impl Eq for SkillString {}
