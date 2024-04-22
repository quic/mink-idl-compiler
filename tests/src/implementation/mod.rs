const TRUTH: crate::interfaces::itest::Collection = crate::interfaces::itest::Collection {
    a: 0,
    b: 1,
    c: 2,
    d: 3,
};

const TRUTH2: crate::interfaces::itest::SingleEncapsulated = crate::interfaces::itest::SingleEncapsulated {
    inner: 0,
};

mod test1;
mod test2;

pub use test1::{ITest1, ITest3};
pub use test2::ITest2;

fn test_singlular_object(
    o: Option<&crate::interfaces::itest1::ITest1>,
) -> Result<(), crate::object::Error> {
    use crate::interfaces::itest::*;
    use qcom_core::object::error::transport::BADOBJ;

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

    assert_eq!(o.well_documented_method(SUCCESS_FLAG), Ok(SUCCESS_FLAG));

    Ok(())
}
