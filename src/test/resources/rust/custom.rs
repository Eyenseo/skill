#![feature(test)]
#![feature(nll)]

extern crate custom;

#[cfg(test)]
#[allow(unused_mut)]
#[allow(non_snake_case)]
#[allow(unused_imports)]
#[allow(unused_variables)]
mod tests {
    extern crate env_logger;
    extern crate failure;

    use custom::common::error::*;
    use custom::common::SkillObject;
    use custom::common::*;
    use custom::SkillFile;
    use custom::*;

    use self::failure::Fail;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::collections::LinkedList;
    use std::rc::Rc;

    struct CleanupApiCustomAcceptCustom;

    impl Drop for CleanupApiCustomAcceptCustom {
        fn drop(&mut self) {
            let _ignore = ::std::fs::remove_file(
                "api_custom_accept_custom_6993977d-76a6-4ea6-9996-861de3589b68.sf",
            );
        }
    }

    #[test]
    fn api_custom_accept_custom() {
        let _logger = env_logger::try_init();
        let _cleanup = CleanupApiCustomAcceptCustom;

        let mut c_id = 0;

        match SkillFile::create("api_custom_accept_custom_6993977d-76a6-4ea6-9996-861de3589b68.sf")
        {
            Ok(mut sf) => match || -> Result<(), SkillFail> {
                sf.check()?;
                // create objects
                let c = sf.custom_mut().add();
                // set fields
                c.borrow_mut().set_any(Some(Box::new("42".to_owned())));
                // assert fields
                assert_eq!(c.borrow().get_any().is_some(), true);
                assert_eq!(c.borrow().get_any().as_ref().unwrap().is::<String>(), true);
                assert_eq!(
                    c.borrow()
                        .get_any()
                        .as_ref()
                        .unwrap()
                        .downcast_ref::<String>()
                        .is_none(),
                    false
                );
                assert_eq!(
                    c.borrow()
                        .get_any()
                        .as_ref()
                        .unwrap()
                        .downcast_ref::<String>()
                        .unwrap(),
                    &"42".to_owned()
                );
                // serialize
                sf.close()?;
                // remember object IDs - type hierarchy makes them difficult to calculate for the generator
                c_id = c.borrow().get_skill_id();
                Ok(())
            }() {
                Ok(_) => {}
                Err(e) => {
                    if let Some(bt) = e.backtrace() {
                        panic!("{}\n{}", e, bt)
                    } else {
                        panic!("{}", e)
                    }
                }
            },
            Err(e) => {
                if let Some(bt) = e.backtrace() {
                    panic!("{}\n{}", e, bt)
                } else {
                    panic!("{}", e)
                }
            }
        };

        match SkillFile::open(
            "api_custom_accept_custom_6993977d-76a6-4ea6-9996-861de3589b68.sf",
            FileMode::R,
        ) {
            Ok(mut sf) => match sf.check() {
                Ok(_) => {
                    // get objects
                    let c = match sf.custom().get(c_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("Object c was not retrieved because:{}", e),
                    };
                    // assert fields
                    assert_eq!(c.borrow().get_any().is_none(), true);
                }
                Err(e) => {
                    if let Some(bt) = e.backtrace() {
                        panic!("{}\n{}", e, bt)
                    } else {
                        panic!("{}", e)
                    }
                }
            },
            Err(e) => {
                if let Some(bt) = e.backtrace() {
                    panic!("{}\n{}", e, bt)
                } else {
                    panic!("{}", e)
                }
            }
        };
    }
}
