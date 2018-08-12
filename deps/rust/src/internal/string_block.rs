use common::error::*;
use common::internal::io::magic::bytes_v64;
use common::internal::io::*;
use common::internal::*;
use common::*;

use std::collections::HashSet;
use std::rc::Rc;

#[derive(Default, Debug)]
pub(crate) struct StringBlock {
    pool: Vec<Rc<SkillString>>,
    set: HashSet<Rc<SkillString>>,
    literal_keeper: LiteralKeeper,
}

// TODO improve user interface
// => split reading to another type
impl StringBlock {
    pub(crate) fn new() -> StringBlock {
        StringBlock {
            pool: Vec::default(),
            set: HashSet::default(),
            literal_keeper: LiteralKeeper::default(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.pool.len()
    }

    fn reserve(&mut self, size: usize) {
        self.pool.reserve(size);
        self.set.reserve(size);
    }
    pub(crate) fn extend(&mut self, size: usize) {
        let reserve = self.pool.len();
        self.reserve(reserve + size);
    }
    fn add_raw(&mut self, s: &str) -> Result<(), SkillFail> {
        let ss = Rc::new(SkillString::new(self.pool.len() + 1, s));
        let ss = if let Some(v) = self.literal_keeper.get(&ss) {
            v.set_skill_id(self.pool.len() + 1)?;
            Ok(v)
        } else {
            if let Some(_) = self.set.get(&ss) {
                return Err(SkillFail::internal(InternalFail::DuplicatedString {
                    string: s.to_owned(),
                }));
            }
            Ok(ss)
        }?;

        self.pool.push(ss.clone());
        self.set.insert(ss);
        Ok(())
    }
    pub(crate) fn add(&mut self, s: &str) -> Rc<SkillString> {
        // this is bad ...
        let v = Rc::new(SkillString::new(self.pool.len() + 1, s));
        if let Some(v) = self.set.get(&v) {
            return v.clone();
        }
        self.pool.push(v.clone());
        self.set.insert(v.clone());
        v
    }
    pub(crate) fn get(&self, i: usize) -> Result<Rc<SkillString>, SkillFail> {
        if i == 0 {
            return Err(SkillFail::user(UserFail::ReservedID { id: 0 }));
        }
        Ok(self.pool[i - 1].clone())
    }

    pub(crate) fn read_string_pool(&mut self, reader: &mut FileReader) -> Result<(), SkillFail> {
        info!(target: "SkillParsing", "~Block Start~");
        let string_amount = reader.read_v64()? as usize; // amount
        info!(target: "SkillParsing", "~Amount: {}", string_amount);
        let mut lengths = Vec::new();
        lengths.reserve(string_amount);
        self.extend(string_amount);

        let mut pre_offset = 0;
        info!(target: "SkillParsing", "~Length block");
        for _ in 0..string_amount {
            // TODO use mmap
            let offset = reader.read_i32()? as u32;
            lengths.push(offset - pre_offset);
            pre_offset = offset;
        }
        info!(target: "SkillParsing", "~Strings");
        for length in lengths {
            let s = reader.read_raw_string(length)?;
            info!(target: "SkillParsing", "~~String: {}", &s);
            self.add_raw(&s)?;
        }
        info!(target: "SkillParsing", "~Block End~");
        Ok(())
    }

    pub(crate) fn finalize(&mut self) -> Result<(), SkillFail> {
        // TODO this shoudl be done on write?
        for s in self.literal_keeper.get_rest() {
            s.set_skill_id(self.pool.len() + 1)?;
            self.pool.push(s.clone());
            self.set.insert(s);
        }
        Ok(())
    }

    pub(crate) fn lit(&self) -> &LiteralKeeper {
        &self.literal_keeper
    }

    pub(crate) fn write_block(&self, writer: &mut FileWriter) -> Result<(), SkillFail> {
        // TODO strings should be pruned/compressed when strong_count is 1
        info!(
            target: "SkillWriting",
            "~String Block Start~"
        );

        let amount = self.pool.len();
        info!(
            target: "SkillWriting",
            "~~Write {} Strings",
            amount
        );
        if amount > 0 {
            let mut lengths = writer.jump(bytes_v64(amount as i64) + amount * 4)?;
            lengths.write_v64(amount as i64)?;

            let mut offset: i32 = 0;
            let mut i = 0;
            for s in self.pool.iter() {
                i += 1;
                offset += s.string().len() as i32;
                writer.write_raw_string(s.as_str())?;
                lengths.write_i32(offset)?;
            }
            // TODO flush async?
            lengths.flush()?;
        } else {
            // tiny optimization
            writer.write_i8(0)?;
        }
        Ok(())
    }
}
