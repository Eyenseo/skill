use common::error::*;
use common::Ptr;

pub(crate) trait ObjectReader {
    fn read_object<T>(&self, index: usize) -> Result<Ptr<T>, SkillFail>;
}
