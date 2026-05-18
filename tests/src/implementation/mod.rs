// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

const TRUTH: crate::interfaces::itest::Collection = crate::interfaces::itest::Collection {
    a: 0,
    b: 1,
    c: 2,
    d: 3,
};

const TRUTH2: crate::interfaces::itest::SingleEncapsulated =
    crate::interfaces::itest::SingleEncapsulated { inner: 0 };

mod test1;
mod test2;

pub use test1::{ITest1, ITest3};
pub use test2::ITest2;

fn test_singular_object(
    o: Option<&crate::interfaces::itest1::ITest1>,
) -> Result<(), crate::object::Error> {
    use crate::interfaces::itest::*;
    use crate::object::error::transport::BADOBJ;

    let Some(o) = o else {
        return Err(BADOBJ);
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
    assert_eq!(o.in_struct(&TRUTH), Ok(()));
    assert_eq!(o.in_small_struct(&TRUTH2), Ok(()));
    assert_eq!(o.add_1000(5), Ok(1005));
    assert_eq!(o.struct_array_in(&[TRUTH, TRUTH]), Ok(()));
    let zeroed = Collection { a: 0, b: 0, c: 0, d: 0 };
    let mut s_out = [zeroed; 2];
    let mut s_out_lenout = 0usize;
    assert_eq!(o.struct_array_out(&mut s_out, &mut s_out_lenout), Ok(()));
    assert_eq!(s_out_lenout, 2);
    assert_eq!(s_out[0], TRUTH);
    assert_eq!(s_out[1], TRUTH);
    let (arr, magic) = o.primitive_array_in_struct().unwrap();
    assert_eq!(arr.a, [7, 8]);
    assert_eq!(arr.c[0], crate::interfaces::itest::F2 { a: 9, b: 7 });
    assert_eq!(arr.c[1], crate::interfaces::itest::F2 { a: 8, b: 9 });
    assert_eq!(arr.d, SUCCESS_FLAG as u16);
    assert_eq!(magic, SUCCESS_FLAG);

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
    assert_eq!(o.out_struct(), Ok(TRUTH));
    assert_eq!(o.out_small_struct(), Ok(TRUTH2));

    assert_eq!(o.well_documented_method(SUCCESS_FLAG), Ok(SUCCESS_FLAG));

    let expected = crate::interfaces::itest1::IDLVersion::new(2,0,0);
    let expected_val: u32 = expected.into();
    assert_eq!(o.api_version(), Ok(expected_val));
    assert_eq!(2, expected.major());
    assert_eq!(0, expected.minor());
    assert_eq!(0, expected.patch());

    Ok(())
}
