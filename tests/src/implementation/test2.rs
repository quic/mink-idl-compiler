use crate::{
    implementation::ITest1,
    interfaces::{
        itest::{ObjInStruct, SUCCESS_FLAG},
        itest2::{self, IITest2},
    },
};

pub struct ITest2;
impl ITest2 {
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl IITest2 for ITest2 {
    fn r#entrypoint(
        &mut self,
        o: Option<&crate::interfaces::itest1::ITest1>,
    ) -> Result<(), itest2::Error> {
        assert_eq!(super::test_singlular_object(o), Ok(()));
        let o = o.unwrap();
        let objects: [Option<crate::interfaces::itest1::ITest1>; 3] = [
            Some(super::ITest1::new(1).into()),
            None,
            Some(super::ITest1::new(2).into()),
        ];
        assert_eq!(o.test_obj_array_in(&objects), Ok(SUCCESS_FLAG));
        let (objects, flag) = o.test_obj_array_out().unwrap();
        assert_eq!(flag, SUCCESS_FLAG);
        for object in objects {
            assert_eq!(super::test_singlular_object(object.as_ref()), Ok(()));
        }

        const VALID_PS: [u32; 4] = [SUCCESS_FLAG; 4];
        let output = o
            .objects_in_struct(&ObjInStruct {
                p1: VALID_PS,
                first_obj: Some(ITest1::new(1).into()),
                p2: VALID_PS,
                should_be_empty: None,
                p3: VALID_PS,
                second_obj: Some(ITest1::new(2).into()),
            })
            .unwrap();
        assert_eq!(output.p1, VALID_PS);
        assert_eq!(output.p2, VALID_PS);
        assert_eq!(output.p3, VALID_PS);
        super::test_singlular_object(output.first_obj.as_ref()).unwrap();
        assert_eq!(output.should_be_empty, None);
        super::test_singlular_object(output.second_obj.as_ref()).unwrap();

        assert_eq!(o.un_implemented(3), Err(crate::object::error::generic::INVALID.into()));

        Ok(())
    }
}
