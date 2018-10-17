use common::error::*;
use common::internal::StringBlock;
use common::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct StringPool {
    string_block: Rc<RefCell<StringBlock>>,
}

impl StringPool {
    pub(crate) fn new(string_block: Rc<RefCell<StringBlock>>) -> StringPool {
        StringPool { string_block }
    }
    pub(crate) fn string_block(&self) -> Rc<RefCell<StringBlock>> {
        self.string_block.clone()
    }

    /// # Arguments
    /// * `s` - to add to the pool
    ///
    /// # Returns
    /// String that equals the given string
    pub fn add(&mut self, s: &str) -> Rc<SkillString> {
        self.string_block.borrow_mut().add(s)
    }

    /// # Arguments
    /// * `id` - of the string to get
    ///
    /// # Returns
    /// String with given id
    pub fn get(&self, id: usize) -> Result<Option<Rc<SkillString>>, SkillFail> {
        self.string_block.borrow().get(id)
    }

    /// Removes all strings that are only kept alive through this class
    pub(crate) fn compress(&mut self) -> Result<(), SkillFail> {
        self.string_block.borrow_mut().compress()
    }

    pub fn extend(&mut self, size: usize) {
        self.string_block.borrow_mut().extend(size);
    }
}
