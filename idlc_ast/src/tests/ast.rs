use crate::Primitive;

#[test]
fn ensure_boundaries() {
    macro_rules! check {
        ($idl_type: literal, $primitive: expr, $ty: ident, $($invalid: expr),*) => {
            assert_eq!(
                Primitive::new($idl_type, &$ty::MAX.to_string()).unwrap(),
                $primitive
            );
            assert_eq!(
                Primitive::new($idl_type, &$ty::MIN.to_string()).unwrap(),
                $primitive
            );
            $(
                Primitive::new($idl_type, $invalid).unwrap_err();
            )*
        };
    }

    macro_rules! check_unsigned {
        ($idl_type: literal, $primitive: expr, $ty: ident) => {
            assert_eq!(
                Primitive::new($idl_type, &format!("{:#X}", $ty::MAX)).unwrap(),
                $primitive
            );
            assert_eq!(
                Primitive::new($idl_type, &format!("{:#X}", $ty::MIN)).unwrap(),
                $primitive
            );
            check!(
                $idl_type,
                $primitive,
                $ty,
                &($ty::MAX as u128 + 1).to_string(),
                "-1",
                "-0x1"
            )
        };
    }
    macro_rules! check_signed {
        ($idl_type: literal, $primitive: expr, $ty: ident) => {
            assert_eq!(
                Primitive::new($idl_type, &format!("{:#X}", $ty::MAX)).unwrap(),
                $primitive,
            );
            assert_eq!(
                Primitive::new($idl_type, &format!("-0x{:X}", $ty::MIN)).unwrap(),
                $primitive
            );
            check!(
                $idl_type,
                $primitive,
                $ty,
                &($ty::MAX as u128 + 1).to_string(),
                &($ty::MIN as i128 - 1).to_string(),
                &format!("{:#X}", $ty::MIN as i128 - 1)
            )
        };
    }

    check_unsigned!("uint8", Primitive::Uint8, u8);
    check_unsigned!("uint16", Primitive::Uint16, u16);
    check_unsigned!("uint32", Primitive::Uint32, u32);
    check_unsigned!("uint64", Primitive::Uint64, u64);

    check_signed!("int8", Primitive::Int8, i8);
    check_signed!("int16", Primitive::Int16, i16);
    check_signed!("int32", Primitive::Int32, i32);
    check_signed!("int64", Primitive::Int64, i64);

    let mut f64_max = f64::MAX.to_string();
    let mut f64_min = f64::MIN.to_string();
    check!("float32", Primitive::Float32, f32, &f64_max, &f64_min);
    f64_max.push('1'); // overflow f64
    f64_min.push('1'); // underflow f64
    check!("float64", Primitive::Float64, f64, &f64_max, &f64_min);
}

#[test]
#[should_panic = "Duplicate attribute"]
fn duplicately_defined_function_attribute() {
    crate::from_string(
        std::path::PathBuf::new(),
        r"interface IFoo {
        #[optional]
        #[optional]
        method tmp();
    };",
        false,
    )
    .unwrap_err();
}
