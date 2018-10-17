#![feature(test)]
#![feature(nll)]

extern crate subtypes;

#[cfg(test)]
#[allow(unused_mut)]
#[allow(non_snake_case)]
#[allow(unused_imports)]
#[allow(unused_variables)]
mod tests {
    extern crate env_logger;
    extern crate failure;

    use subtypes::common::error::*;
    use subtypes::common::SkillObject;
    use subtypes::common::*;
    use subtypes::SkillFile;
    use subtypes::*;

    use self::failure::Fail;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::collections::LinkedList;
    use std::rc::Rc;

    struct CleanupApiSubtypesAcceptDelete;

    impl Drop for CleanupApiSubtypesAcceptDelete {
        fn drop(&mut self) {
            let _ignore = ::std::fs::remove_file(
                "api_subtypes_accept_delete_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
            );
        }
    }

    #[test]
    fn api_subtypes_accept_delete() {
        let _logger = env_logger::try_init();
        let _cleanup = CleanupApiSubtypesAcceptDelete;

        let mut a_2_id = 0;
        let mut b_id = 0;
        let mut c_id = 0;
        let mut d_id = 0;

        match SkillFile::create(
            "api_subtypes_accept_delete_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
        ) {
            Ok(mut sf) => match || -> Result<(), SkillFail> {
                sf.check()?;
                // create objects
                let a_1 = sf.a_mut().add();
                let a_2 = sf.a_mut().add();
                let b = sf.b_mut().add();
                let c = sf.c_mut().add();
                let d = sf.d_mut().add();
                // set fields
                a_2.borrow_mut()
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
                // assert fields
                assert_eq!(a_1.borrow_mut().get_a().is_none(), true);
                assert_eq!(a_2.borrow_mut().get_a().is_some(), true);
                assert_eq!(
                    a_2.borrow_mut()
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
                    d.clone().cast::<SkillObject>(),
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
                // DELETE
                sf.delete_strong(a_1)?;
                // serialize
                sf.close()?;
                // remember object IDs - type hierarchy makes them difficult to calculate for the generator
                a_2_id = a_2.borrow().get_skill_id();
                b_id = b.borrow().get_skill_id();
                c_id = c.borrow().get_skill_id();
                d_id = d.borrow().get_skill_id();
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

        // NOTE a_1 is deleted which would have been 1 so now a_2 is 1
        assert_eq!(a_2_id, 1);

        match SkillFile::open(
            "api_subtypes_accept_delete_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
            FileMode::R,
        ) {
            Ok(mut sf) => match sf.check() {
                Ok(_) => {
                    // get objects
                    let a_2 = match sf.a().get(a_2_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("Object a was not retrieved because:{}", e),
                    };
                    let b = match sf.b().get(b_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("Object b was not retrieved because:{}", e),
                    };
                    let c = match sf.c().get(c_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("Object c was not retrieved because:{}", e),
                    };
                    let d = match sf.d().get(d_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("Object d was not retrieved because:{}", e),
                    };
                    // assert fields
                    assert_eq!(a_2.borrow_mut().get_a().is_some(), true);
                    assert_eq!(
                        a_2.borrow_mut()
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
                        d.clone().cast::<SkillObject>(),
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

    struct CleanupApiSubtypesRejectUseAfterDelete;

    impl Drop for CleanupApiSubtypesRejectUseAfterDelete {
        fn drop(&mut self) {
            let _ignore = ::std::fs::remove_file(
                "api_subtypes_reject_use_after_delete_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
            );
        }
    }

    #[test]
    fn api_subtypes_reject_use_after_delete() {
        let _logger = env_logger::try_init();
        let _cleanup = CleanupApiSubtypesRejectUseAfterDelete;

        match SkillFile::create(
            "api_subtypes_reject_use_after_delete_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
        ) {
            Ok(mut sf) => match || -> Result<(), SkillFail> {
                sf.check()?;
                // create objects
                let a = sf.a_mut().add();
                // set fields
                a.borrow_mut()
                    .set_a(Some(a.clone().cast::<A>().unwrap().downgrade()));
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
                    a.clone().cast::<SkillObject>(),
                );
                // DELETE
                // NOTE this should panic as a has a reference to itself serialize
                // This tests the new instance count
                match sf.delete(a.downgrade()) {
                    Ok(_) => panic!("This delete should be forbidden"),
                    Err(_) => {}
                }
                // serialize
                sf.close()?;
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
    }

    struct CleanupApiSubtypesRejectUseAfterDeleteAfterWrite;

    impl Drop for CleanupApiSubtypesRejectUseAfterDeleteAfterWrite {
        fn drop(&mut self) {
            let _ignore = ::std::fs::remove_file(
                "api_subtypes_reject_use_after_delete_after_write_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
            );
        }
    }

    #[test]
    fn api_subtypes_reject_use_after_delete_after_write() {
        let _logger = env_logger::try_init();
        let _cleanup = CleanupApiSubtypesRejectUseAfterDeleteAfterWrite;

        match SkillFile::create(
            "api_subtypes_reject_use_after_delete_after_write_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
        ) {
            Ok(mut sf) => match || -> Result<(), SkillFail> {
                sf.check()?;
                // create objects
                let a = sf.a_mut().add();
                // set fields
                a.borrow_mut().set_a(Some(a.clone().cast::<A>().unwrap().downgrade()));
                // assert fields
                assert_eq!(a.borrow_mut().get_a().is_some(), true);
                assert_eq!(
                    a.borrow_mut()
                        .get_a()
                        .as_ref()
                        .unwrap().upgrade().unwrap()
                        .cast::<SkillObject>(),
                    a.clone().cast::<SkillObject>(),
                );
                // serialize
                sf.compress()?;
                // DELETE
                // NOTE this should panic as a has a reference to itself serialize
                // This tests the static instance count after writing a new instance
                match sf.delete(a.downgrade()) {
                    Ok(_) => panic!("This delete should be forbidden"),
                    Err(_) => {}
                }
                // serialize
                sf.close()?;
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

    struct CleanupApiSubtypesRejectUseAfterDeleteAfterRead;

    impl Drop for CleanupApiSubtypesRejectUseAfterDeleteAfterRead {
        fn drop(&mut self) {
            let _ignore = ::std::fs::remove_file(
                "api_subtypes_reject_use_after_delete_after_read_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
            );
        }
    }

    #[test]
    fn api_subtypes_reject_use_after_delete_after_read() {
        let _logger = env_logger::try_init();
        let _cleanup = CleanupApiSubtypesRejectUseAfterDeleteAfterRead;

        let mut a_id = 0;

        match SkillFile::create(
            "api_subtypes_reject_use_after_delete_after_read_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
        ) {
            Ok(mut sf) => match || -> Result<(), SkillFail> {
                sf.check()?;
                // create objects
                let a = sf.a_mut().add();
                // set fields
                a.borrow_mut().set_a(Some(a.clone().cast::<A>().unwrap().downgrade()));
                // assert fields
                assert_eq!(a.borrow_mut().get_a().is_some(), true);
                assert_eq!(
                    a.borrow_mut()
                        .get_a()
                        .as_ref()
                        .unwrap().upgrade().unwrap()
                        .cast::<SkillObject>(),
                    a.clone().cast::<SkillObject>(),
                );
                // serialize
                sf.close()?;
                // remember object IDs - type hierarchy makes them difficult to calculate for the generator
                a_id = a.borrow().get_skill_id();
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

        match SkillFile::open(
            "api_subtypes_reject_use_after_delete_after_read_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
            FileMode::R) {
            Ok(mut sf) => match sf.check() {
                Ok(_) => {
                    // get objects
                    let a = match sf.a().get(a_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("Object a was not retrieved because:{}", e),
                    };
                    // assert fields
                    assert_eq!(a.borrow_mut().get_a().is_some(), true);
                    assert_eq!(
                        a.borrow_mut()
                            .get_a()
                            .as_ref()
                            .unwrap().upgrade().unwrap()
                            .cast::<SkillObject>(),
                        a.clone().cast::<SkillObject>(),
                    );
                    // DELETE
                    // NOTE this should panic as a has a reference to itself serialize
                    // This tests the static instance count after writing a new instance
                    match sf.delete(a.cast::<SkillObject>().unwrap().downgrade()) {
                        Ok(_) => panic!("This delete should be forbidden"),
                        Err(_) => {}
                    }
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

    struct CleanupApiSubtypesAcceptForceDelete;

    impl Drop for CleanupApiSubtypesAcceptForceDelete {
        fn drop(&mut self) {
            let _ignore = ::std::fs::remove_file(
                "api_subtypes_accept_force_delete_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
            );
        }
    }

    #[test]
    fn api_subtypes_accept_force_delete() {
        let _logger = env_logger::try_init();
        let _cleanup = CleanupApiSubtypesAcceptForceDelete;

        let mut a_id = 0;
        let mut b_id = 0;
        let mut c_id = 0;
        let mut d_id = 0;

        match SkillFile::create(
            "api_subtypes_accept_force_delete_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
        ) {
            Ok(mut sf) => match || -> Result<(), SkillFail> {
                sf.check()?;
                // create objects
                let a = sf.a_mut().add();
                let b = sf.b_mut().add();
                let c = sf.c_mut().add();
                let d = sf.d_mut().add();
                // set fields
                a.borrow_mut()
                    .set_a(Some(a.clone().cast::<A>().unwrap().downgrade()));
                b.borrow_mut()
                    .set_a(Some(a.clone().cast::<A>().unwrap().downgrade()));
                b.borrow_mut()
                    .set_b(Some(b.clone().cast::<B>().unwrap().downgrade()));
                c.borrow_mut()
                    .set_a(Some(a.clone().cast::<A>().unwrap().downgrade()));
                c.borrow_mut()
                    .set_c(Some(c.clone().cast::<C>().unwrap().downgrade()));
                d.borrow_mut()
                    .set_a(Some(a.clone().cast::<A>().unwrap().downgrade()));
                d.borrow_mut()
                    .set_b(Some(b.clone().cast::<B>().unwrap().downgrade()));
                d.borrow_mut()
                    .set_d(Some(d.clone().cast::<D>().unwrap().downgrade()));
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
                    a.clone().cast::<SkillObject>(),
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
                    a.clone().cast::<SkillObject>(),
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
                    b.clone().cast::<SkillObject>(),
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
                    a.clone().cast::<SkillObject>(),
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
                    a.clone().cast::<SkillObject>(),
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
                    b.clone().cast::<SkillObject>(),
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
                // DELETE
                sf.delete_force(a.clone().downgrade());
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

        assert_eq!(b_id, 1); // NOTE a is not around anymore so b should have id 1

        match SkillFile::open(
            "api_subtypes_accept_force_delete_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
            FileMode::R,
        ) {
            Ok(mut sf) => match sf.check() {
                Ok(_) => {
                    // get objects
                    let b = match sf.b().get(b_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("Object b was not retrieved because:{}", e),
                    };
                    let c = match sf.c().get(c_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("Object c was not retrieved because:{}", e),
                    };
                    let d = match sf.d().get(d_id) {
                        Ok(ptr) => ptr,
                        Err(e) => panic!("Object d was not retrieved because:{}", e),
                    };
                    // assert fields
                    assert_eq!(b.borrow_mut().get_a().is_none(), true);
                    assert_eq!(b.borrow_mut().get_b().is_some(), true);
                    assert_eq!(
                        b.borrow_mut()
                            .get_b()
                            .as_ref()
                            .unwrap()
                            .upgrade()
                            .unwrap()
                            .cast::<SkillObject>(),
                        b.clone().cast::<SkillObject>(),
                    );
                    assert_eq!(c.borrow_mut().get_a().is_none(), true);
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
                    assert_eq!(d.borrow_mut().get_a().is_none(), true);
                    assert_eq!(d.borrow_mut().get_b().is_some(), true);
                    assert_eq!(
                        d.borrow_mut()
                            .get_b()
                            .as_ref()
                            .unwrap()
                            .upgrade()
                            .unwrap()
                            .cast::<SkillObject>(),
                        b.clone().cast::<SkillObject>(),
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

    struct CleanupApiSubtypesAcceptMemoryFreed;

    impl Drop for CleanupApiSubtypesAcceptMemoryFreed {
        fn drop(&mut self) {
            let _ignore = ::std::fs::remove_file(
                "api_subtypes_accept_memory_freed_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
            );
        }
    }

    #[test]
    fn api_subtypes_accept_memory_freed() {
        let _logger = env_logger::try_init();
        let _cleanup = CleanupApiSubtypesAcceptMemoryFreed;

        match SkillFile::create(
            "api_subtypes_accept_memory_freed_74c1be11-af16-4f2a-bcd4-d26f43267bb7.sf",
        ) {
            Ok(mut sf) => match || -> Result<(), SkillFail> {
                sf.check()?;
                let a = {
                    // create objects
                    let a = sf.a_mut().add();
                    // set fields
                    a.borrow_mut()
                        .set_a(Some(a.clone().cast::<A>().unwrap().downgrade()));
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
                        a.clone().cast::<SkillObject>(),
                    );
                    // DELETE
                    sf.delete_force(a.clone().downgrade());
                    // serialize
                    sf.close()?;
                    a.downgrade()
                };
                assert_eq!(a.upgrade(), None); // NOTE should not be around anymore
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
    }
}
