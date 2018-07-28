use common::error::SkillError;
use common::internal::LiteralKeeper;
use common::internal::SkillObject;
use common::io::base_writer::bytes_v64;
use common::io::FileReader;
use common::io::FileWriter;
use common::SkillString;

use std::collections::HashSet;
use std::rc::Rc;

#[derive(Default, Debug)]
pub struct StringBlock {
    pool: Vec<Rc<SkillString>>,
    set: HashSet<Rc<SkillString>>,
    literal_keeper: LiteralKeeper,
}
// TODO improve user interface
// => split reading to another type
impl StringBlock {
    pub fn new() -> StringBlock {
        StringBlock {
            pool: Vec::default(),
            set: HashSet::default(),
            literal_keeper: LiteralKeeper::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    fn reserve(&mut self, size: usize) {
        self.pool.reserve(size);
        self.set.reserve(size);
    }
    fn extend(&mut self, size: usize) {
        let reserve = self.pool.len();
        self.reserve(reserve + size);
        self.reserve(reserve + size);
    }
    fn add_raw(&mut self, s: &str) {
        let ss = Rc::new(SkillString::new(self.pool.len(), s));
        let ss = if let Some(v) = self.literal_keeper.get(&ss) {
            v.set_skill_id(self.pool.len());
            v
        } else {
            if let Some(_) = self.set.get(&ss) {
                panic!("String |{}| was already contained in the pool", s);
            }
            ss
        };

        self.pool.push(ss.clone());
        self.set.insert(ss);
    }
    pub fn add(&mut self, s: &str) -> Rc<SkillString> {
        // this is bad ...
        let v = Rc::new(SkillString::new(self.pool.len(), s));
        if let Some(v) = self.set.get(&v) {
            return v.clone();
        }
        self.pool.push(v.clone());
        self.set.insert(v.clone());
        v
    }
    // TODO replace with Rc -- beter suited for the job
    pub fn get(&self, i: usize) -> Rc<SkillString> {
        if i == 0 {
            panic!("StringBlock index starts at 1 not 0");
        }
        self.pool[i - 1].clone()
    }

    pub fn read_string_block(&mut self, reader: &mut FileReader) -> Result<(), SkillError> {
        info!(target:"SkillParsing", "~Block Start~");
        let string_amount = reader.read_v64()? as usize; // amount
        info!(target:"SkillParsing", "~Amount: {}", string_amount);
        let mut lengths = Vec::new();
        lengths.reserve(string_amount);
        self.extend(string_amount);

        let mut pre_offset = 0;
        info!(target:"SkillParsing","~Length block");
        for _ in 0..string_amount {
            // TODO use mmap
            let offset = reader.read_i32()? as u32;
            lengths.push(offset - pre_offset);
            pre_offset = offset;
        }
        info!(target:"SkillParsing", "~Strings");
        for length in lengths {
            let s = reader.read_raw_string(length)?;
            info!(target:"SkillParsing", "~~String: {}", &s);
            self.add_raw(&s);
        }
        info!(target:"SkillParsing", "~Block End~");
        Ok(())
    }

    pub fn finalize(&mut self) {
        // TODO this shoudl be done on write?
        for s in self.literal_keeper.get_rest() {
            s.set_skill_id(self.pool.len() + 1);
            self.pool.push(s.clone());
            self.set.insert(s);
        }
    }

    pub fn lit(&self) -> &LiteralKeeper {
        &self.literal_keeper
    }

    pub fn write_block(&self, writer: &mut FileWriter) -> Result<(), SkillError> {
        // TODO strings should be pruned/compressed when strong_count is 1

        let amount = self.pool.len();
        if amount > 0 {
            let mut lengths = writer.jump(bytes_v64(amount as i64) + amount * 4)?;
            lengths.write_v64(amount as i64);

            let mut offset: i32 = 0;
            for s in self.pool.iter() {
                offset += s.string().len() as i32;
                writer.write_raw_string(s.as_str());
                lengths.write_i32(offset);
            }
            // TODO flush async?
            lengths.flush();
        } else {
            // tiny optimization
            writer.write_i8(0);
        }
        Ok(())
    }
}
