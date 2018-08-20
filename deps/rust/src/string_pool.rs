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

    pub fn add(&mut self, s: &str) -> Rc<SkillString> {
        self.string_block.borrow_mut().add(s)
    }
    pub fn get(&self, i: usize) -> Result<Option<Rc<SkillString>>, SkillFail> {
        self.string_block.borrow().get(i)
    }
    pub fn extend(&mut self, size: usize) {
        self.string_block.borrow_mut().extend(size);
    }
}
