use common::error::SkillError;
use common::io::FileReader;
use common::ptr::Ptr;

#[derive(Default, Debug)]
pub struct StringBlock {
    pool: Vec<Ptr<String>>, // TODO add set to filter dupliates -> Box the string
}

impl StringBlock {
    pub fn new() -> StringBlock {
        StringBlock { pool: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn reserve(&mut self, size: usize) {
        self.pool.reserve(size);
    }
    pub fn extend(&mut self, size: usize) {
        let reserve = self.pool.len();
        self.reserve(reserve + size);
    }
    pub fn add(&mut self, s: &str) -> usize {
        self.pool.push(Ptr::new(String::from(s)));
        self.pool.len()
    }
    // TODO replace with Rc -- beter suited for the job
    pub fn get(&self, i: usize) -> Ptr<String> {
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
            self.add(&s);
        }
        info!(target:"SkillParsing", "~Block End~");
        Ok(())
    }
}
