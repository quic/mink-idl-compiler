use super::TRUTH;
use qcom_core::object::error::transport::BADOBJ;

use crate::interfaces::itest::{SingleEncapsulated, SUCCESS_FLAG};
use crate::interfaces::itest2::{self, IITest2};

pub struct ITest2;
impl ITest2 {
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl IITest2 for ITest2 {
    fn test_f2(&mut self, _f: &crate::interfaces::itest::F1) -> Result<(), itest2::Error> {
        Err(itest2::MY_CUSTOM_ERROR)
    }

    fn test_obj_in(
        &mut self,
        o: Option<&crate::interfaces::itest1::ITest1>,
    ) -> Result<u32, itest2::Error> {
        let Some(o) = o else {
            return Err(BADOBJ.into());
        };
        assert_eq!(o.single_in(SUCCESS_FLAG), Ok(()));
        assert_eq!(
            o.single_primitive_in(&[], &mut [], &mut 0, SUCCESS_FLAG),
            Ok(())
        );
        assert_eq!(
            o.multiple_primitive(
                &[],
                &mut [],
                &mut 0,
                SUCCESS_FLAG as u16,
                None,
                SUCCESS_FLAG,
                &mut [],
                &mut 0
            ),
            Ok((SUCCESS_FLAG as u16, None, SUCCESS_FLAG as u64))
        );
        assert_eq!(
            o.bundled_with_unbundled(
                &SingleEncapsulated {
                    inner: SUCCESS_FLAG
                },
                SUCCESS_FLAG,
                &TRUTH
            ),
            Ok(())
        );
        assert_eq!(
            o.primitive_plus_struct_in(
                &SingleEncapsulated {
                    inner: SUCCESS_FLAG,
                },
                SUCCESS_FLAG
            ),
            Ok(())
        );

        assert_eq!(o.single_out(), Ok(SUCCESS_FLAG));
        assert_eq!(
            o.single_primitive_out(&[], &mut [], &mut 0),
            Ok(SUCCESS_FLAG)
        );
        assert_eq!(
            o.primitive_plus_struct_out(),
            Ok((
                SingleEncapsulated {
                    inner: SUCCESS_FLAG,
                },
                SUCCESS_FLAG
            ))
        );

        Ok(SUCCESS_FLAG)
    }

    fn test_obj_out(&mut self) -> Result<Option<crate::interfaces::itest1::ITest1>, itest2::Error> {
        Ok(Some(super::ITest1::new(96).into()).into())
    }

    fn test_bundle(
        &mut self,
        _xxx: &[u8],
        _yyy: &mut [u8],
        yyy_lenout: &mut usize,
        a: u32,
        _xxx1: &[u8],
        b: u8,
        c: u32,
    ) -> Result<(u32, u16, u32), itest2::Error> {
        *yyy_lenout = 0;
        Ok((a + 10, (b + 20) as u16, c + 30))
    }

    fn test_array(
        &mut self,
        f_in: &[crate::interfaces::itest::F1],
        f_out: &mut [crate::interfaces::itest::F1],
        f_out_lenout: &mut usize,
        _f_y: &crate::interfaces::itest::F1,
        _a: &[u32],
        _b: &mut [u32],
        _b_lenout: &mut usize,
        d: i16,
    ) -> Result<(crate::interfaces::itest::F1, i32), itest2::Error> {
        if f_in.len() <= f_out.len() {
            f_out.iter_mut().zip(f_in).for_each(|(o, i)| o.a = i.a + 5);
            *f_out_lenout = f_in.len();
            return Ok((
                crate::interfaces::itest::F1 {
                    a: f_in.len() as u32,
                },
                d as i32,
            ));
        }
        Err(crate::object::error::generic::INVALID.into())
    }
}
