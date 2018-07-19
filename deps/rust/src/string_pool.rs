use common::error::SkillError;
use common::internal::LiteralKeeper;
use common::io::FileReader;

use std::collections::HashSet;
use std::rc::Rc;

#[derive(Default, Debug)]
pub struct StringBlock {
    pool: Vec<Rc<String>>, // TODO add set to filter dupliates -> Box the string
    set: HashSet<Rc<String>>,
    literal_keeper: LiteralKeeper,
}

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
        if let Some(v) = self.set.get(&String::from(s)) {
            panic!("String |{}| was already contained in the pool", s);
        }
        let v = if let Some(v) = self.literal_keeper.get(s) {
            v
        } else {
            Rc::new(String::from(s))
        };
        self.pool.push(v.clone());
        self.set.insert(v);
    }
    pub fn add(&mut self, s: &str) -> Rc<String> {
        if let Some(v) = self.set.get(&String::from(s)) {
            return v.clone();
        }
        let v = Rc::new(String::from(s));
        self.pool.push(v.clone());
        self.set.insert(v.clone());
        v
    }
    // TODO replace with Rc -- beter suited for the job
    pub fn get(&self, i: usize) -> Rc<String> {
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
        for s in self.literal_keeper.get_rest() {
            self.pool.push(s.clone());
            self.set.insert(s);
        }
    }

    pub fn lit(&self) -> &LiteralKeeper {
        &self.literal_keeper
    }
}
