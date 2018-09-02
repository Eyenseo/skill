#![feature(test)]

extern crate subtypes;
extern crate unknown;

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(unused_imports)]
#[allow(unused_variables)]
mod tests {
    extern crate env_logger;
    extern crate failure;

    use self::failure::Fail;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::collections::LinkedList;
    use std::rc::Rc;

    struct CleanupApiSubtypesUnknownSubtypes;

    impl Drop for CleanupApiSubtypesUnknownSubtypes {
        fn drop(&mut self) {
            let _ignore = ::std::fs::remove_file(
                "/tmp/api_unknown_accept_subtypes_unknown_subtypes_8578bb69-5cc4-466d-93b5-beb823b6299a.sf",
            );
        }
    }

    #[test]
    fn subtypes_create_write_unknown_read_write_subtypes_read() {
        let _logger = env_logger::try_init();
        let _cleanup = CleanupApiSubtypesUnknownSubtypes;

        let mut a_id = 0;
        let mut b_id = 0;
        let mut c_id = 0;
        let mut d_id = 0;
        {
            use subtypes::common::error::*;
            use subtypes::common::*;
            use subtypes::*;

            match SkillFile::create(
                "/tmp/api_unknown_accept_subtypes_unknown_subtypes_8578bb69-5cc4-466d-93b5-beb823b6299a.sf",
            ) {
                Ok(sf) => match || -> Result<(), SkillFail> {
                    sf.check()?;
                    // create objects
                    let a = sf.a_mut().add();
                    let b = sf.b_mut().add();
                    let c = sf.c_mut().add();
                    let d = sf.d_mut().add();
                    // set fields
                    a.borrow_mut()
                        .set_a(Some(d.clone().cast::<A>().unwrap().downgrade()));
                    b.borrow_mut()
                        .set_a(Some(d.clone().cast::<A>().unwrap().downgrade()));
                    b.borrow_mut()
                        .set_b(Some(d.clone().cast::<B>().unwrap().downgrade()));
                    c.borrow_mut()
                        .set_a(Some(d.clone().cast::<A>().unwrap().downgrade()));
                    c.borrow_mut()
                        .set_c(Some(c.clone().cast::<C>().unwrap().downgrade()));
                    d.borrow_mut()
                        .set_a(Some(d.clone().cast::<A>().unwrap().downgrade()));
                    d.borrow_mut()
                        .set_b(Some(d.clone().cast::<B>().unwrap().downgrade()));
                    d.borrow_mut()
                        .set_d(Some(d.clone().cast::<D>().unwrap().downgrade()));
                    // serialize
                    sf.close()?;
                    // remember object IDs - type hierarchy makes them difficult to calculate for the generator
                    a_id = a.borrow().get_skill_id();
                    b_id = b.borrow().get_skill_id();
                    c_id = c.borrow().get_skill_id();
                    d_id = d.borrow().get_skill_id();
                    Ok(())
                }() {
                    Ok(_) => {}
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
            };
        }
        {
            use unknown::common::error::*;
            use unknown::common::*;
            use unknown::*;

            match SkillFile::open(
                "/tmp/api_unknown_accept_subtypes_unknown_subtypes_8578bb69-5cc4-466d-93b5-beb823b6299a.sf",
                FileMode::RW) {
                Ok(sf) => match || -> Result<(), SkillFail> {
                    sf.check()?;
                    // get objects
                    let a = match sf.a().get(a_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("ObjectProper a was not retrieved because:{}", e),
                    };
                    let c = match sf.c().get(c_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("ObjectProper c was not retrieved because:{}", e),
                    };
                    // assert fields
                    assert_eq!(a.borrow_mut().get_a().is_some(), true);
                    assert_eq!(c.borrow_mut().get_a().is_some(), true);
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
            };
        }
        {
            use subtypes::common::error::*;
            use subtypes::common::*;
            use subtypes::*;

            match SkillFile::open(
                "/tmp/api_unknown_accept_subtypes_unknown_subtypes_8578bb69-5cc4-466d-93b5-beb823b6299a.sf",
                FileMode::R   ) {
                Ok(sf) => match sf.check() {
                    Ok(_) => {
                        // get objects
                        let a = match sf.a().get(a_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("ObjectProper a was not retrieved because:{}", e),
                        };
                        let b = match sf.b().get(b_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("ObjectProper b was not retrieved because:{}", e),
                        };
                        let c = match sf.c().get(c_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("ObjectProper c was not retrieved because:{}", e),
                        };
                        let d = match sf.d().get(d_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("ObjectProper d was not retrieved because:{}", e),
                        };
                        // assert fields
                        assert_eq!(a.borrow_mut().get_a().is_some(), true);
                        assert_eq!(
                            a.borrow_mut()
                                .get_a()
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .cast::<SkillObject>(),
                            d.clone().cast::<SkillObject>(),
                        );
                        assert_eq!(b.borrow_mut().get_a().is_some(), true);
                        assert_eq!(
                            b.borrow_mut()
                                .get_a()
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .cast::<SkillObject>(),
                            d.clone().cast::<SkillObject>(),
                        );
                        assert_eq!(b.borrow_mut().get_b().is_some(), true);
                        assert_eq!(
                            b.borrow_mut()
                                .get_b()
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .cast::<SkillObject>(),
                            d.clone().cast::<SkillObject>(),
                        );
                        assert_eq!(c.borrow_mut().get_a().is_some(), true);
                        assert_eq!(
                            c.borrow_mut()
                                .get_a()
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .cast::<SkillObject>(),
                            d.cast::<SkillObject>(),
                        );
                        assert_eq!(c.borrow_mut().get_c().is_some(), true);
                        assert_eq!(
                            c.borrow_mut()
                                .get_c()
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .cast::<SkillObject>(),
                            c.clone().cast::<SkillObject>(),
                        );
                        assert_eq!(d.borrow_mut().get_a().is_some(), true);
                        assert_eq!(
                            d.borrow_mut()
                                .get_a()
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .cast::<SkillObject>(),
                            d.clone().cast::<SkillObject>(),
                        );
                        assert_eq!(d.borrow_mut().get_b().is_some(), true);
                        assert_eq!(
                            d.borrow_mut()
                                .get_b()
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .cast::<SkillObject>(),
                            d.clone().cast::<SkillObject>(),
                        );
                        assert_eq!(d.borrow_mut().get_d().is_some(), true);
                        assert_eq!(
                            d.borrow_mut()
                                .get_d()
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .cast::<SkillObject>(),
                            d.clone().cast::<SkillObject>(),
                        );
                    }
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
            };
        }
    }
}
