#![feature(test)]
#![feature(nll)]

extern crate basic_types;
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

    struct CleanupApiBasicTypesAcceptAll;

    impl Drop for CleanupApiBasicTypesAcceptAll {
        fn drop(&mut self) {
            let _ignore = ::std::fs::remove_file("/tmp/api_basic_types_undefined_basic_types_28ab2b82-9815-4ef7-8806-7a0d23ccccad.sf");
        }
    }

    #[test]
    fn api_basic_types_undefined_basic_types() {
        let _logger = env_logger::try_init();
        let _cleanup = CleanupApiBasicTypesAcceptAll;

        let mut all_id = 0;
        let mut all_aUserType_obj_int64I_obj_id = 0;
        let mut all_anotherUserType_obj_id = 0;
        let mut all_aUserType_obj_id = 0;
        let mut all_aUserType_obj_int32_obj_id = 0;
        let mut all_anotherUserType_obj_float32_obj_id = 0;
        let mut all_aUserType_obj_int64V_obj_id = 0;
        let mut all_anotherUserType_obj_float64_obj_id = 0;
        let mut all_aUserType_obj_int8_obj_id = 0;
        let mut all_aUserType_obj_int16_obj_id = 0;
        let mut all_aBool_obj_id = 0;
        let mut all_aString_obj_id = 0;
        {
            use basic_types::common::error::*;
            use basic_types::common::internal::SkillObject;
            use basic_types::common::*;
            use basic_types::skill_file::*;
            use basic_types::*;

            match SkillFile::create("/tmp/api_basic_types_undefined_basic_types_28ab2b82-9815-4ef7-8806-7a0d23ccccad.sf") {
                Ok(sf) => match || -> Result<(), SkillFail> {
                    sf.check()?;
                    // create objects
                    let all = sf.basic_types.borrow_mut().add();
                    let all_aUserType_obj_int64I_obj = sf.basic_int64i.borrow_mut().add();
                    let all_anotherUserType_obj = sf.basic_floats.borrow_mut().add();
                    let all_aUserType_obj = sf.basic_integers.borrow_mut().add();
                    let all_aUserType_obj_int32_obj = sf.basic_int32.borrow_mut().add();
                    let all_anotherUserType_obj_float32_obj = sf.basic_float32.borrow_mut().add();
                    let all_aUserType_obj_int64V_obj = sf.basic_int64v.borrow_mut().add();
                    let all_anotherUserType_obj_float64_obj = sf.basic_float64.borrow_mut().add();
                    let all_aUserType_obj_int8_obj = sf.basic_int8.borrow_mut().add();
                    let all_aUserType_obj_int16_obj = sf.basic_int16.borrow_mut().add();
                    let all_aBool_obj = sf.basic_bool.borrow_mut().add();
                    let all_aString_obj = sf.basic_string.borrow_mut().add();
                    // set fields
                    all.borrow_mut().set_a_user_type(Some(all_aUserType_obj.clone().nucast::<BasicIntegersT>().unwrap()));
                    all.borrow_mut().set_a_string(Some(all_aString_obj.clone().nucast::<BasicStringT>().unwrap()));
                    all.borrow_mut().set_a_list({
                        let mut list: LinkedList<f32> = LinkedList::default();
                        list.push_back(3 as f32);
                        list.push_back(4 as f32);
                        list
                    });
                    all.borrow_mut().set_a_map({
                        let mut map: HashMap<i16, i8> = HashMap::default();
                        map.reserve(1);
                        map.insert(5 as i16, 6 as i8);
                        map
                    });
                    all.borrow_mut().set_an_array({
                        let mut vec: Vec<Option<Ptr<BasicIntegersT>>> = Vec::default();
                        vec.reserve(1);
                        vec.push(Some(all_aUserType_obj.clone().nucast::<BasicIntegersT>().unwrap()));
                        vec
                    });
                    all.borrow_mut().set_an_annotation(Some(all_aBool_obj.clone().nucast::<SkillObject>().unwrap()));
                    all.borrow_mut().set_another_user_type(Some(all_anotherUserType_obj.clone().nucast::<BasicFloatsT>().unwrap()));
                    all.borrow_mut().set_a_set({
                        let mut set: HashSet<i8> = HashSet::default();
                        set.reserve(1);
                        set.insert(2 as i8);
                        set
                    });
                    all.borrow_mut().set_a_bool(Some(all_aBool_obj.clone().nucast::<BasicBoolT>().unwrap()));
                    all_aUserType_obj_int64I_obj.borrow_mut().set_basic_int(0 as i64);
                    all_anotherUserType_obj.borrow_mut().set_float32(Some(all_anotherUserType_obj_float32_obj.clone().nucast::<BasicFloat32T>().unwrap()));
                    all_anotherUserType_obj.borrow_mut().set_float64(Some(all_anotherUserType_obj_float64_obj.clone().nucast::<BasicFloat64T>().unwrap()));
                    all_aUserType_obj.borrow_mut().set_int32(Some(all_aUserType_obj_int32_obj.clone().nucast::<BasicInt32T>().unwrap()));
                    all_aUserType_obj.borrow_mut().set_int8(Some(all_aUserType_obj_int8_obj.clone().nucast::<BasicInt8T>().unwrap()));
                    all_aUserType_obj.borrow_mut().set_int64v(Some(all_aUserType_obj_int64V_obj.clone().nucast::<BasicInt64VT>().unwrap()));
                    all_aUserType_obj.borrow_mut().set_int64i(Some(all_aUserType_obj_int64I_obj.clone().nucast::<BasicInt64IT>().unwrap()));
                    all_aUserType_obj.borrow_mut().set_int16(Some(all_aUserType_obj_int16_obj.clone().nucast::<BasicInt16T>().unwrap()));
                    all_aUserType_obj_int32_obj.borrow_mut().set_basic_int(-1 as i32);
                    all_anotherUserType_obj_float32_obj.borrow_mut().set_basic_float(1 as f32);
                    all_aUserType_obj_int64V_obj.borrow_mut().set_basic_int(1 as i64);
                    all_anotherUserType_obj_float64_obj.borrow_mut().set_basic_float(2 as f64);
                    all_aUserType_obj_int8_obj.borrow_mut().set_basic_int(-3 as i8);
                    all_aUserType_obj_int16_obj.borrow_mut().set_basic_int(-2 as i16);
                    all_aBool_obj.borrow_mut().set_basic_bool(true);
                    all_aString_obj.borrow_mut().set_basic_string({
                        let mut sp = sf.strings.borrow_mut();
                        let s = sp.add("Hello World!");
                        s
                    });
                    // serialize
                    sf.close()?;
                    // remember object IDs - type hierarchy makes them difficult to calculate for the generator
                    all_id = all.borrow().get_skill_id();
                    all_aUserType_obj_int64I_obj_id = all_aUserType_obj_int64I_obj.borrow().get_skill_id();
                    all_anotherUserType_obj_id = all_anotherUserType_obj.borrow().get_skill_id();
                    all_aUserType_obj_id = all_aUserType_obj.borrow().get_skill_id();
                    all_aUserType_obj_int32_obj_id = all_aUserType_obj_int32_obj.borrow().get_skill_id();
                    all_anotherUserType_obj_float32_obj_id = all_anotherUserType_obj_float32_obj.borrow().get_skill_id();
                    all_aUserType_obj_int64V_obj_id = all_aUserType_obj_int64V_obj.borrow().get_skill_id();
                    all_anotherUserType_obj_float64_obj_id = all_anotherUserType_obj_float64_obj.borrow().get_skill_id();
                    all_aUserType_obj_int8_obj_id = all_aUserType_obj_int8_obj.borrow().get_skill_id();
                    all_aUserType_obj_int16_obj_id = all_aUserType_obj_int16_obj.borrow().get_skill_id();
                    all_aBool_obj_id = all_aBool_obj.borrow().get_skill_id();
                    all_aString_obj_id = all_aString_obj.borrow().get_skill_id();
                    Ok(())
                }() {
                    Ok(_) => {}
                    Err(e) => if let Some(bt) = e.backtrace() {
                        panic!("{}\n{}", e, bt)
                    } else {
                        panic!("{}", e)
                    }
                },
                Err(e) => if let Some(bt) = e.backtrace() {
                    panic!("{}\n{}", e, bt)
                } else {
                    panic!("{}", e)
                },
            };

            match SkillFile::open("/tmp/api_basic_types_undefined_basic_types_28ab2b82-9815-4ef7-8806-7a0d23ccccad.sf") {
                Ok(sf) => match sf.check() {
                    Ok(_) => {
                        // get objects
                        let all = match sf.basic_types.borrow().get(all_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj_int64I_obj = match sf.basic_int64i.borrow().get(all_aUserType_obj_int64I_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj_int64I_obj was not retrieved because:{}", e),
                        };
                        let all_anotherUserType_obj = match sf.basic_floats.borrow().get(all_anotherUserType_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_anotherUserType_obj was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj = match sf.basic_integers.borrow().get(all_aUserType_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj_int32_obj = match sf.basic_int32.borrow().get(all_aUserType_obj_int32_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj_int32_obj was not retrieved because:{}", e),
                        };
                        let all_anotherUserType_obj_float32_obj = match sf.basic_float32.borrow().get(all_anotherUserType_obj_float32_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_anotherUserType_obj_float32_obj was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj_int64V_obj = match sf.basic_int64v.borrow().get(all_aUserType_obj_int64V_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj_int64V_obj was not retrieved because:{}", e),
                        };
                        let all_anotherUserType_obj_float64_obj = match sf.basic_float64.borrow().get(all_anotherUserType_obj_float64_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_anotherUserType_obj_float64_obj was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj_int8_obj = match sf.basic_int8.borrow().get(all_aUserType_obj_int8_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj_int8_obj was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj_int16_obj = match sf.basic_int16.borrow().get(all_aUserType_obj_int16_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj_int16_obj was not retrieved because:{}", e),
                        };
                        let all_aBool_obj = match sf.basic_bool.borrow().get(all_aBool_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aBool_obj was not retrieved because:{}", e),
                        };
                        let all_aString_obj = match sf.basic_string.borrow().get(all_aString_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aString_obj was not retrieved because:{}", e),
                        };
                        // assert fields
                        assert_eq!(
                            all.borrow_mut().get_a_user_type().is_some(), true);
                        assert_eq!(
                            all.borrow_mut().get_a_user_type().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj.clone().nucast::<BasicIntegersT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all.borrow_mut().get_a_string().is_some(), true);
                        assert_eq!(
                            all.borrow_mut().get_a_string().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aString_obj.clone().nucast::<BasicStringT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(*all.borrow_mut().get_a_list(), {
                            let mut list: LinkedList<f32> = LinkedList::default();
                            list.push_back(3 as f32);
                            list.push_back(4 as f32);
                            list
                        });
                        assert_eq!(*all.borrow_mut().get_a_map(), {
                            let mut map: HashMap<i16, i8> = HashMap::default();
                            map.reserve(1);
                            map.insert(5 as i16, 6 as i8);
                            map
                        });
                        assert_eq!(*all.borrow_mut().get_an_array(), {
                            let mut vec: Vec<Option<Ptr<BasicIntegersT>>> = Vec::default();
                            vec.reserve(1);
                            vec.push(Some(all_aUserType_obj.clone().nucast::<BasicIntegersT>().unwrap()));
                            vec
                        });
                        assert_eq!(
                            all.borrow_mut().get_an_annotation().is_some(), true);
                        assert_eq!(
                            all.borrow_mut().get_an_annotation().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aBool_obj.clone().nucast::<SkillObject>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all.borrow_mut().get_another_user_type().is_some(), true);
                        assert_eq!(
                            all.borrow_mut().get_another_user_type().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_anotherUserType_obj.clone().nucast::<BasicFloatsT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(*all.borrow_mut().get_a_set(), {
                            let mut set: HashSet<i8> = HashSet::default();
                            set.reserve(1);
                            set.insert(2 as i8);
                            set
                        });
                        assert_eq!(
                            all.borrow_mut().get_a_bool().is_some(), true);
                        assert_eq!(
                            all.borrow_mut().get_a_bool().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aBool_obj.clone().nucast::<BasicBoolT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(all_aUserType_obj_int64I_obj.borrow_mut().get_basic_int(), 0 as i64);
                        assert_eq!(
                            all_anotherUserType_obj.borrow_mut().get_float32().is_some(), true);
                        assert_eq!(
                            all_anotherUserType_obj.borrow_mut().get_float32().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_anotherUserType_obj_float32_obj.clone().nucast::<BasicFloat32T>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_anotherUserType_obj.borrow_mut().get_float64().is_some(), true);
                        assert_eq!(
                            all_anotherUserType_obj.borrow_mut().get_float64().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_anotherUserType_obj_float64_obj.clone().nucast::<BasicFloat64T>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int32().is_some(), true);
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int32().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj_int32_obj.clone().nucast::<BasicInt32T>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int8().is_some(), true);
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int8().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj_int8_obj.clone().nucast::<BasicInt8T>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int64v().is_some(), true);
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int64v().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj_int64V_obj.clone().nucast::<BasicInt64VT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int64i().is_some(), true);
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int64i().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj_int64I_obj.clone().nucast::<BasicInt64IT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int16().is_some(), true);
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int16().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj_int16_obj.clone().nucast::<BasicInt16T>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(all_aUserType_obj_int32_obj.borrow_mut().get_basic_int(), -1 as i32);
                        assert_eq!(all_anotherUserType_obj_float32_obj.borrow_mut().get_basic_float(), 1 as f32);
                        assert_eq!(all_aUserType_obj_int64V_obj.borrow_mut().get_basic_int(), 1 as i64);
                        assert_eq!(all_anotherUserType_obj_float64_obj.borrow_mut().get_basic_float(), 2 as f64);
                        assert_eq!(all_aUserType_obj_int8_obj.borrow_mut().get_basic_int(), -3 as i8);
                        assert_eq!(all_aUserType_obj_int16_obj.borrow_mut().get_basic_int(), -2 as i16);
                        assert_eq!(all_aBool_obj.borrow_mut().get_basic_bool(), true);
                        assert_eq!(*all_aString_obj.borrow_mut().get_basic_string(), {
                            let mut sp = sf.strings.borrow_mut();
                            let s = sp.add("Hello World!");
                            s
                        });
                    }
                    Err(e) => if let Some(bt) = e.backtrace() {
                        panic!("{}\n{}", e, bt)
                    } else {
                        panic!("{}", e)
                    }
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
            use unknown::common::internal::SkillObject;
            use unknown::skill_file::SkillFile;
            use unknown::*;

            match SkillFile::open(
                "/tmp/api_basic_types_undefined_basic_types_28ab2b82-9815-4ef7-8806-7a0d23ccccad.sf",
            ) {
                Ok(sf) => match || -> Result<(), SkillFail> {
                    sf.check()?;
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
            use basic_types::common::error::*;
            use basic_types::common::internal::SkillObject;
            use basic_types::common::*;
            use basic_types::skill_file::*;
            use basic_types::*;

            match SkillFile::open("/tmp/api_basic_types_undefined_basic_types_28ab2b82-9815-4ef7-8806-7a0d23ccccad.sf") {
                Ok(sf) => match sf.check() {
                    Ok(_) => {
                        // get objects
                        let all = match sf.basic_types.borrow().get(all_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj_int64I_obj = match sf.basic_int64i.borrow().get(all_aUserType_obj_int64I_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj_int64I_obj was not retrieved because:{}", e),
                        };
                        let all_anotherUserType_obj = match sf.basic_floats.borrow().get(all_anotherUserType_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_anotherUserType_obj was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj = match sf.basic_integers.borrow().get(all_aUserType_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj_int32_obj = match sf.basic_int32.borrow().get(all_aUserType_obj_int32_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj_int32_obj was not retrieved because:{}", e),
                        };
                        let all_anotherUserType_obj_float32_obj = match sf.basic_float32.borrow().get(all_anotherUserType_obj_float32_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_anotherUserType_obj_float32_obj was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj_int64V_obj = match sf.basic_int64v.borrow().get(all_aUserType_obj_int64V_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj_int64V_obj was not retrieved because:{}", e),
                        };
                        let all_anotherUserType_obj_float64_obj = match sf.basic_float64.borrow().get(all_anotherUserType_obj_float64_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_anotherUserType_obj_float64_obj was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj_int8_obj = match sf.basic_int8.borrow().get(all_aUserType_obj_int8_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj_int8_obj was not retrieved because:{}", e),
                        };
                        let all_aUserType_obj_int16_obj = match sf.basic_int16.borrow().get(all_aUserType_obj_int16_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aUserType_obj_int16_obj was not retrieved because:{}", e),
                        };
                        let all_aBool_obj = match sf.basic_bool.borrow().get(all_aBool_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aBool_obj was not retrieved because:{}", e),
                        };
                        let all_aString_obj = match sf.basic_string.borrow().get(all_aString_obj_id) {
                            Ok(ptr) => ptr,
                            Err(e) => panic!("Object all_aString_obj was not retrieved because:{}", e),
                        };
                        // assert fields
                        assert_eq!(
                            all.borrow_mut().get_a_user_type().is_some(), true);
                        assert_eq!(
                            all.borrow_mut().get_a_user_type().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj.clone().nucast::<BasicIntegersT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all.borrow_mut().get_a_string().is_some(), true);
                        assert_eq!(
                            all.borrow_mut().get_a_string().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aString_obj.clone().nucast::<BasicStringT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(*all.borrow_mut().get_a_list(), {
                            let mut list: LinkedList<f32> = LinkedList::default();
                            list.push_back(3 as f32);
                            list.push_back(4 as f32);
                            list
                        });
                        assert_eq!(*all.borrow_mut().get_a_map(), {
                            let mut map: HashMap<i16, i8> = HashMap::default();
                            map.reserve(1);
                            map.insert(5 as i16, 6 as i8);
                            map
                        });
                        assert_eq!(*all.borrow_mut().get_an_array(), {
                            let mut vec: Vec<Option<Ptr<BasicIntegersT>>> = Vec::default();
                            vec.reserve(1);
                            vec.push(Some(all_aUserType_obj.clone().nucast::<BasicIntegersT>().unwrap()));
                            vec
                        });
                        assert_eq!(
                            all.borrow_mut().get_an_annotation().is_some(), true);
                        assert_eq!(
                            all.borrow_mut().get_an_annotation().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aBool_obj.clone().nucast::<SkillObject>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all.borrow_mut().get_another_user_type().is_some(), true);
                        assert_eq!(
                            all.borrow_mut().get_another_user_type().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_anotherUserType_obj.clone().nucast::<BasicFloatsT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(*all.borrow_mut().get_a_set(), {
                            let mut set: HashSet<i8> = HashSet::default();
                            set.reserve(1);
                            set.insert(2 as i8);
                            set
                        });
                        assert_eq!(
                            all.borrow_mut().get_a_bool().is_some(), true);
                        assert_eq!(
                            all.borrow_mut().get_a_bool().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aBool_obj.clone().nucast::<BasicBoolT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(all_aUserType_obj_int64I_obj.borrow_mut().get_basic_int(), 0 as i64);
                        assert_eq!(
                            all_anotherUserType_obj.borrow_mut().get_float32().is_some(), true);
                        assert_eq!(
                            all_anotherUserType_obj.borrow_mut().get_float32().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_anotherUserType_obj_float32_obj.clone().nucast::<BasicFloat32T>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_anotherUserType_obj.borrow_mut().get_float64().is_some(), true);
                        assert_eq!(
                            all_anotherUserType_obj.borrow_mut().get_float64().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_anotherUserType_obj_float64_obj.clone().nucast::<BasicFloat64T>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int32().is_some(), true);
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int32().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj_int32_obj.clone().nucast::<BasicInt32T>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int8().is_some(), true);
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int8().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj_int8_obj.clone().nucast::<BasicInt8T>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int64v().is_some(), true);
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int64v().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj_int64V_obj.clone().nucast::<BasicInt64VT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int64i().is_some(), true);
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int64i().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj_int64I_obj.clone().nucast::<BasicInt64IT>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int16().is_some(), true);
                        assert_eq!(
                            all_aUserType_obj.borrow_mut().get_int16().as_ref().unwrap().nucast::<SkillObject>(),
                            Some(all_aUserType_obj_int16_obj.clone().nucast::<BasicInt16T>().unwrap()).unwrap().nucast::<SkillObject>(),
                        );
                        assert_eq!(all_aUserType_obj_int32_obj.borrow_mut().get_basic_int(), -1 as i32);
                        assert_eq!(all_anotherUserType_obj_float32_obj.borrow_mut().get_basic_float(), 1 as f32);
                        assert_eq!(all_aUserType_obj_int64V_obj.borrow_mut().get_basic_int(), 1 as i64);
                        assert_eq!(all_anotherUserType_obj_float64_obj.borrow_mut().get_basic_float(), 2 as f64);
                        assert_eq!(all_aUserType_obj_int8_obj.borrow_mut().get_basic_int(), -3 as i8);
                        assert_eq!(all_aUserType_obj_int16_obj.borrow_mut().get_basic_int(), -2 as i16);
                        assert_eq!(all_aBool_obj.borrow_mut().get_basic_bool(), true);
                        assert_eq!(*all_aString_obj.borrow_mut().get_basic_string(), {
                            let mut sp = sf.strings.borrow_mut();
                            let s = sp.add("Hello World!");
                            s
                        });
                    }
                    Err(e) => if let Some(bt) = e.backtrace() {
                        panic!("{}\n{}", e, bt)
                    } else {
                        panic!("{}", e)
                    }
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
