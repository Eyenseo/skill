#![feature(test)]

extern crate unknown;

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(unused_imports)]
#[allow(unused_variables)]
mod tests {
    extern crate env_logger;
    extern crate failure;

    use unknown::common::error::*;
    use unknown::common::internal::SkillObject;
    use unknown::common::*;
    use unknown::skill_file::*;
    use unknown::*;

    use self::failure::Fail;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::collections::LinkedList;
    use std::fs;
    use std::rc::Rc;

    struct CleanupApiUnknownReadWriteRead;

    impl Drop for CleanupApiUnknownReadWriteRead {
        fn drop(&mut self) {
            let _ignore = ::std::fs::remove_file(
                "/tmp/api_unknown_accept_read_write_read_8578bb69-5cc4-466d-93b5-beb823b6299a.sf",
            );
        }
    }

    #[test]
    fn unknown_read_write_read_check() {
        let _logger = env_logger::try_init();
        let _cleanup = CleanupApiUnknownReadWriteRead;

        match fs::copy(
            "../../../src/test/resources/genbinary/[[empty]]/accept/localBasePoolOffset.sf",
            "/tmp/api_unknown_accept_read_write_read_8578bb69-5cc4-466d-93b5-beb823b6299a.sf",
        ) {
            Ok(_) => {}
            Err(e) => panic!("Unable to copy test file!"),
        }

        let a_id = 1;
        let c_id = 6;

        match SkillFile::open(
            "/tmp/api_unknown_accept_read_write_read_8578bb69-5cc4-466d-93b5-beb823b6299a.sf",
        ) {
            Ok(sf) => match || -> Result<(), SkillFail> {
                sf.check()?;
                // get objects
                let a = match sf.a.borrow().get(a_id) {
                    Ok(ptr) => ptr,
                    Err(e) => panic!("Object a was not retrieved because:{}", e),
                };
                let c = match sf.c.borrow().get(c_id) {
                    Ok(ptr) => ptr,
                    Err(e) => panic!("Object c was not retrieved because:{}", e),
                };
                // assert fields
                assert_eq!(a.borrow_mut().get_a().is_some(), true);
                assert_eq!(
                    a.borrow_mut()
                        .get_a()
                        .as_ref()
                        .unwrap()
                        .nucast::<SkillObject>(),
                    Some(a.clone().nucast::<AT>().unwrap())
                        .unwrap()
                        .nucast::<SkillObject>(),
                );
                assert_eq!(c.borrow_mut().get_a().is_some(), true);
                assert_eq!(
                    c.borrow_mut()
                        .get_a()
                        .as_ref()
                        .unwrap()
                        .nucast::<SkillObject>(),
                    Some(c.clone().nucast::<AT>().unwrap())
                        .unwrap()
                        .nucast::<SkillObject>(),
                );
                sf.close()?;
                Ok(())
            }() {
                Ok(_) => (),
                Err(e) => if let Some(bt) = e.backtrace() {
                    panic!("{}\n{}", e, bt)
                } else {
                    panic!("{}", e)
                },
            },
            Err(e) => if let Some(bt) = e.backtrace() {
                panic!("{}\n{}", e, bt)
            } else {
                panic!("{}", e)
            },
        }

        let a_id = 1;
        let c_id = 12; // NOTE Blocks were merged

        match SkillFile::open(
            "/tmp/api_unknown_accept_read_write_read_8578bb69-5cc4-466d-93b5-beb823b6299a.sf",
        ) {
            Ok(sf) => match || -> Result<(), SkillFail> {
                sf.check()?;
                // get objects
                let a = match sf.a.borrow().get(a_id) {
                    Ok(ptr) => ptr,
                    Err(e) => panic!("Object a was not retrieved because:{}", e),
                };
                let c = match sf.c.borrow().get(c_id) {
                    Ok(ptr) => ptr,
                    Err(e) => panic!("Object c was not retrieved because:{}", e),
                };
                // assert fields
                assert_eq!(a.borrow_mut().get_a().is_some(), true);
                assert_eq!(
                    a.borrow_mut()
                        .get_a()
                        .as_ref()
                        .unwrap()
                        .nucast::<SkillObject>(),
                    Some(a.clone().nucast::<AT>().unwrap())
                        .unwrap()
                        .nucast::<SkillObject>(),
                );
                assert_eq!(c.borrow_mut().get_a().is_some(), true);
                assert_eq!(
                    c.borrow_mut()
                        .get_a()
                        .as_ref()
                        .unwrap()
                        .nucast::<SkillObject>(),
                    Some(c.clone().nucast::<AT>().unwrap())
                        .unwrap()
                        .nucast::<SkillObject>(),
                );
                sf.close()?;
                Ok(())
            }() {
                Ok(_) => (),
                Err(e) => if let Some(bt) = e.backtrace() {
                    panic!("{}\n{}", e, bt)
                } else {
                    panic!("{}", e)
                },
            },
            Err(e) => if let Some(bt) = e.backtrace() {
                panic!("{}\n{}", e, bt)
            } else {
                panic!("{}", e)
            },
        }
    }
}
