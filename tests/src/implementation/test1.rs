// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use super::{TRUTH, TRUTH2};
use crate::interfaces::itest::{self, ArrInStruct, SingleEncapsulated, F2, SUCCESS_FLAG};
use crate::interfaces::itest1::{self, IITest1};
use crate::interfaces::itest3::{self, IITest3};

macro_rules! generate_itest1_impl {
    ($impl: ident) => {
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $impl {
            pub value: u32,
        }

        impl $impl {
            #[inline]
            pub const fn new(value: u32) -> Self {
                Self { value }
            }
        }

        impl IITest1 for $impl {
            fn test_f1(&mut self, a: u32) -> Result<u32, itest1::Error> {
                Ok(self.value + a + 1000)
            }

            fn in_struct(
                &mut self,
                input: &crate::interfaces::itest::r#Collection,
            ) -> Result<(), itest1::Error> {
                if input == &TRUTH {
                    Ok(())
                } else {
                    dbg!(input);
                    Err(itest1::MISMATCH)
                }
            }

            fn r#out_struct(
                &mut self,
            ) -> Result<crate::interfaces::itest::r#Collection, itest1::Error> {
                Ok(TRUTH)
            }

            fn in_small_struct(
                &mut self,
                input: &crate::interfaces::itest::r#SingleEncapsulated,
            ) -> Result<(), itest1::Error> {
                if input == &TRUTH2 {
                    Ok(())
                } else {
                    dbg!(input);
                    Err(itest1::MISMATCH)
                }
            }

            fn r#out_small_struct(
                &mut self,
            ) -> Result<crate::interfaces::itest::r#SingleEncapsulated, itest1::Error> {
                Ok(TRUTH2)
            }

            fn r#single_out(&mut self) -> Result<u32, itest1::Error> {
                Ok(0xdead)
            }

            fn r#single_in(&mut self, r#input: u32) -> Result<(), itest1::Error> {
                if input == 0xdead {
                    Ok(())
                } else {
                    dbg!(input);
                    Err(itest1::MISMATCH)
                }
            }

            fn r#single_primitive_in(
                &mut self,
                _: &[u8],
                _: &mut [u8],
                _: &mut usize,
                r#input: u32,
            ) -> Result<(), itest1::Error> {
                self.single_in(r#input)
            }

            fn r#single_primitive_out(
                &mut self,
                _: &[u8],
                _: &mut [u8],
                _: &mut usize,
            ) -> Result<u32, itest1::Error> {
                self.single_out()
            }

            fn r#multiple_primitive(
                &mut self,
                _: &[u8],
                _: &mut [u8],
                _: &mut usize,
                r#input: u16,
                _: Option<&crate::object::Object>,
                r#input2: u32,
                _: &mut [u8],
                _: &mut usize,
            ) -> Result<(u16, Option<crate::object::Object>, u64), itest1::Error> {
                if !(input == itest::SUCCESS_FLAG as u16 && input2 == itest::SUCCESS_FLAG) {
                    dbg!(input, input2);
                    Err(itest1::MISMATCH)
                } else {
                    Ok((SUCCESS_FLAG as u16, None, SUCCESS_FLAG as u64))
                }
            }

            fn r#primitive_plus_struct_in(
                &mut self,
                r#encapsulated: &SingleEncapsulated,
                r#magic: u32,
            ) -> Result<(), itest1::Error> {
                if encapsulated.inner == itest::SUCCESS_FLAG && magic == itest::SUCCESS_FLAG {
                    Ok(())
                } else {
                    dbg!(encapsulated, magic);
                    Err(itest1::MISMATCH)
                }
            }

            fn r#primitive_plus_struct_out(
                &mut self,
            ) -> Result<(SingleEncapsulated, u32), itest1::Error> {
                Ok((
                    SingleEncapsulated {
                        inner: SUCCESS_FLAG,
                    },
                    SUCCESS_FLAG,
                ))
            }

            fn r#primitive_array_in_struct(&mut self) -> Result<(ArrInStruct, u32), itest1::Error> {
                Ok((
                    ArrInStruct {
                        a: [7, 8],
                        c: [F2 { a: 9, b: 7 }, F2 { a: 8, b: 9 }],
                        d: 7,
                    },
                    SUCCESS_FLAG,
                ))
            }

            fn r#bundled_with_unbundled(
                &mut self,
                r#bundled: &crate::interfaces::itest::r#SingleEncapsulated,
                r#magic: u32,
                r#unbundled: &crate::interfaces::itest::r#Collection,
            ) -> Result<(), itest1::Error> {
                if bundled.inner == SUCCESS_FLAG && magic == SUCCESS_FLAG && unbundled == &TRUTH {
                    Ok(())
                } else {
                    dbg!(bundled, magic, unbundled);
                    Err(itest1::MISMATCH)
                }
            }

            fn r#struct_array_in(
                &mut self,
                r#s_in: &[crate::interfaces::itest::r#Collection],
            ) -> Result<(), itest1::Error> {
                for s in s_in.iter() {
                    if s != &TRUTH {
                        dbg!(s);
                        return Err(itest1::MISMATCH);
                    }
                }
                Ok(())
            }

            fn r#struct_array_out(
                &mut self,
                r#s_out: &mut [crate::interfaces::itest::r#Collection],
                r#s_out_lenout: &mut usize,
            ) -> Result<(), itest1::Error> {
                for s in s_out.iter_mut() {
                    *s = TRUTH;
                }
                *s_out_lenout = s_out.len();
                Ok(())
            }

            fn r#well_documented_method(&mut self, r#foo: u32) -> Result<u32, itest1::Error> {
                if foo == SUCCESS_FLAG {
                    Ok(SUCCESS_FLAG)
                } else {
                    Err(itest1::MISMATCH)
                }
            }

            fn test_obj_array_in(
                &mut self,
                o_in: &[Option<crate::interfaces::itest1::ITest1>; 3],
            ) -> Result<u32, itest1::Error> {
                for o in o_in.iter().filter(|o| o.is_some()) {
                    assert_eq!(super::test_singlular_object(o.as_ref()), Ok(()));
                }

                Ok(SUCCESS_FLAG)
            }

            fn r#test_obj_array_out(
                &mut self,
            ) -> Result<([Option<crate::interfaces::itest1::ITest1>; 3], u32), itest1::Error> {
                Ok((
                    [
                        Some(super::ITest1::new(0).into()),
                        Some(super::ITest1::new(1).into()),
                        Some(super::ITest1::new(2).into()),
                    ],
                    SUCCESS_FLAG,
                ))
            }

            fn r#objects_in_struct(
                &mut self,
                r#input: &crate::interfaces::itest::r#ObjInStruct,
            ) -> Result<crate::interfaces::itest::r#ObjInStruct, itest1::Error> {
                assert_eq!(
                    super::test_singlular_object(input.first_obj.as_ref()),
                    Ok(())
                );
                assert_eq!(
                    super::test_singlular_object(input.second_obj.as_ref()),
                    Ok(())
                );
                assert!(input.should_be_empty.is_none());

                assert!(input.p1.iter().all(|x| *x == SUCCESS_FLAG));
                assert!(input.p1.iter().all(|x| *x == SUCCESS_FLAG));
                assert!(input.p1.iter().all(|x| *x == SUCCESS_FLAG));

                Ok(crate::interfaces::itest::ObjInStruct {
                    p1: [SUCCESS_FLAG; 4],
                    first_obj: Some(super::ITest1::new(1).into()),
                    p2: [SUCCESS_FLAG; 4],
                    should_be_empty: None,
                    p3: [SUCCESS_FLAG; 4],
                    second_obj: Some(super::ITest1::new(2).into()),
                })
            }
        }
    };
}

generate_itest1_impl!(ITest1);

generate_itest1_impl!(ITest3);
impl IITest3 for ITest3 {
    fn r#extra_test3(&mut self) -> Result<u32, itest3::Error> {
        Ok(SUCCESS_FLAG)
    }
}
