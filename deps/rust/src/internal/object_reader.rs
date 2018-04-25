use common::Ptr;
use common::SkillError;

pub trait ObjectReader {
    fn read_object<T>(&self, index: usize) -> Result<Ptr<T>, SkillError>;
}
