// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

#include <object.h>
#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

#include "ITest.hpp"
#include "ITest_invoke.hpp"

#include "ITest3.hpp"
#include "ITest3_invoke.hpp"

namespace c {
#include "../c/header.h"
}


extern "C" {
Object create_cpp_itest1(uint32_t value);
}

class ITest1Impl : public ITest1ImplBase {
public:
  ITest1Impl(struct c::CTest1 ctest) { this->ctest = ctest; }
  ~ITest1Impl() {}

  int32_t test_f1(uint32_t a_val, uint32_t *b_ptr) {
    return c::itest1_test_f1(&this->ctest, a_val, b_ptr);
  }
  int32_t in_struct(const Collection &input_ref) {
    return c::itest1_in_struct(&this->ctest, (const c::Collection *)(&input_ref));
  }
  int32_t out_struct(Collection &output_ref) {
    return c::itest1_out_struct(&this->ctest, (c::Collection *)(&output_ref));
  }
  int32_t in_small_struct(const SingleEncapsulated &input_ref) {
    return c::itest1_in_small_struct(&this->ctest, (const c::SingleEncapsulated *)(&input_ref));
  }
  int32_t out_small_struct(SingleEncapsulated &output_ref) {
    return c::itest1_out_small_struct(&this->ctest, (c::SingleEncapsulated *)(&output_ref));
  }
  int32_t single_out(uint32_t *output_ptr) {
    return c::itest1_single_out(&this->ctest, output_ptr);
  }
  int32_t single_in(uint32_t input_val) {
    return c::itest1_single_in(&this->ctest, input_val);
  }
  int32_t single_primitive_in(const void *unused_ptr, size_t unused_len,
                              void *unused2_ptr, size_t unused2_len,
                              size_t *unused2_lenout, uint32_t input_val) {
    return c::itest1_single_primitive_in(&this->ctest, unused_ptr, unused_len,
                                      unused2_ptr, unused2_len, unused2_lenout,
                                      input_val);
  }
  int32_t single_primitive_out(const void *unused_ptr, size_t unused_len,
                               void *unused2_ptr, size_t unused2_len,
                               size_t *unused2_lenout, uint32_t *output_ptr) {
    return c::itest1_single_primitive_out(&this->ctest, unused_ptr, unused_len,
                                       unused2_ptr, unused2_len, unused2_lenout,
                                       output_ptr);
  }
  int32_t multiple_primitive(const void *unused_ptr, size_t unused_len,
                             void *unused2_ptr, size_t unused2_len,
                             size_t *unused2_lenout, uint16_t input_val,
                             uint16_t *output_ptr, const ProxyBase &unused3_ref,
                             ProxyBase &unused4_ref, uint32_t input2_val,
                             uint64_t *output2_ptr, void *unused5_ptr,
                             size_t unused5_len, size_t *unused5_lenout) {
    Object unused4_val = unused4_ref.get();

    CHECK_OK(c::itest1_multiple_primitive(
        &this->ctest, unused_ptr, unused_len, unused2_ptr, unused2_len,
        unused2_lenout, input_val, output_ptr, unused3_ref.get(), &unused4_val,
        input2_val, output2_ptr, unused5_ptr, unused5_len, unused5_lenout));

    unused4_ref = ProxyBase(unused4_val);
    return Object_OK;
  }

  int32_t
  primitive_plus_struct_in(const SingleEncapsulated &encapsulated_ref,
                           uint32_t magic_val) {
    return c::itest1_primitive_plus_struct_in(
        &this->ctest, (const c::SingleEncapsulated *)&encapsulated_ref, magic_val);
  }
  int32_t primitive_plus_struct_out(SingleEncapsulated &encapsulated_ref,
                                    uint32_t *magic_ptr) {
    return c::itest1_primitive_plus_struct_out(
        &this->ctest, (c::SingleEncapsulated *)&encapsulated_ref, magic_ptr);
  }
  int32_t primitive_array_in_struct(ArrInStruct &arr_ref,
                                    uint32_t *magic_ptr) {
    return c::itest1_primitive_array_in_struct(
        &this->ctest, (c::ArrInStruct *)&arr_ref, magic_ptr);
  }
  int32_t bundled_with_unbundled(const SingleEncapsulated &bundled_ref,
                                 uint32_t magic_val,
                                 const Collection &unbundled_ref) {
    return c::itest1_bundled_with_unbundled(
        &this->ctest, (const c::SingleEncapsulated *)&bundled_ref, magic_val,
        (const c::Collection *)&unbundled_ref);
  }
  int32_t struct_array_in(const Collection *s_in_ptr, size_t s_in_len) {
    return itest1_struct_array_in(
        &this->ctest, (const c::Collection *)s_in_ptr, s_in_len);
  }
  int32_t struct_array_out( Collection *s_out_ptr, size_t s_out_len, size_t *s_out_lenout) {
    return c::itest1_struct_array_out(
        &this->ctest, (c::Collection *)s_out_ptr, s_out_len, s_out_lenout);
  }
  int32_t well_documented_method(uint32_t foo_val, uint32_t *bar_ptr) {
    return c::itest1_well_documented_method_real(&this->ctest, foo_val, bar_ptr);
  }
  int32_t test_obj_array_in(const ITest1 (&o_in_ptr)[3], uint32_t *a_ptr) {
    for (size_t i = 0; i < 3; i++) {
      Object o = (o_in_ptr)[i].get();
      if (!Object_isNull(o)) {
        CHECK_OK(c::test_singular_object(o));
      }
    }
    *a_ptr = SUCCESS_FLAG;
    return Object_OK;
  }
  int32_t test_obj_array_out(ITest1 (&out_ref)[3], uint32_t *a_ptr) {
    (out_ref)[0] = create_cpp_itest1(0);
    (out_ref)[1] = create_cpp_itest1(1);
    (out_ref)[2] = create_cpp_itest1(2);
    *a_ptr = SUCCESS_FLAG;
    return Object_OK;
  }
  int32_t objects_in_struct(const ObjInStruct &input_ref,
                            ObjInStruct &output_ref) {
    CHECK_OK(c::test_singular_object(input_ref.first_obj));
    ASSERT(Object_isNull(input_ref.should_be_empty));
    CHECK_OK(c::test_singular_object(input_ref.second_obj));

    for (size_t i = 0; i < sizeof(input_ref.p1) / sizeof(input_ref.p1[0]);
         i++) {
      ASSERT(input_ref.p1[i] == SUCCESS_FLAG);
      ASSERT(input_ref.p2[i] == SUCCESS_FLAG);
      ASSERT(input_ref.p3[i] == SUCCESS_FLAG);

      output_ref.p1[i] = SUCCESS_FLAG;
      output_ref.p2[i] = SUCCESS_FLAG;
      output_ref.p3[i] = SUCCESS_FLAG;
    }
    output_ref.first_obj = create_cpp_itest1(1);
    output_ref.second_obj = create_cpp_itest1(2);
    output_ref.should_be_empty = Object_NULL;
    return Object_OK;
  }

private:
  struct c::CTest1 ctest;
};

extern "C" {

Object create_cpp_itest1(uint32_t value) {
  struct c::CTest1 ctest = {.refs = 1, .value = value};
  ITest1Impl *me = new ITest1Impl(ctest);
  if (me == nullptr) {
    return Object_NULL;
  }

  return (Object){ImplBase::invoke, me};
}
}

class ITest2Impl : public ITest2ImplBase {
public:
  int32_t entrypoint(const ITest1 &o) {
    struct c::CTest1 ctest = {.refs = 1, .value = 1};
    ITest1Impl me(ctest);
    const Object itest1 = o.get();
    ASSERT(!Object_isNull(itest1));
    CHECK_OK(c::test_singular_object(itest1));

    ITest1 objects[3] = {create_cpp_itest1(1), Object_NULL,
                              create_cpp_itest1(2)};
    ITest1 objects_out[3] = {Object_NULL, Object_NULL, Object_NULL};
    uint32_t a = 0;
    CHECK_OK(me.test_obj_array_in(objects, &a));
    ASSERT(a == SUCCESS_FLAG);
    a = 0;
    CHECK_OK(me.test_obj_array_out(objects_out, &a));
    ASSERT(a == SUCCESS_FLAG);

    for (size_t i = 0; i < sizeof(objects) / sizeof(objects[0]); i++) {

      CHECK_OK(c::test_singular_object(objects_out[i].get()));
    }

    const uint32_t VALID_PS[4] = {SUCCESS_FLAG, SUCCESS_FLAG, SUCCESS_FLAG,
                                  SUCCESS_FLAG};

    ObjInStruct input_struct = {
        .first_obj = create_cpp_itest1(1),
        .should_be_empty = Object_NULL,
        .second_obj = create_cpp_itest1(2),
    };
    memcpy(&input_struct.p1, VALID_PS, sizeof(VALID_PS));
    memcpy(&input_struct.p2, VALID_PS, sizeof(VALID_PS));
    memcpy(&input_struct.p3, VALID_PS, sizeof(VALID_PS));
    ObjInStruct output_struct{};
    CHECK_OK(me.objects_in_struct(input_struct, output_struct));
    ASSERT(memcmp(&output_struct.p1, VALID_PS, sizeof(VALID_PS)) == 0);
    ASSERT(memcmp(&output_struct.p2, VALID_PS, sizeof(VALID_PS)) == 0);
    ASSERT(memcmp(&output_struct.p3, VALID_PS, sizeof(VALID_PS)) == 0);

    CHECK_OK(c::test_singular_object(output_struct.first_obj));
    CHECK_OK(c::test_singular_object(output_struct.second_obj));
    ASSERT(Object_isNull(output_struct.should_be_empty));

    Object_ASSIGN_NULL(input_struct.first_obj);
    Object_ASSIGN_NULL(input_struct.second_obj);
    Object_ASSIGN_NULL(input_struct.should_be_empty);

    Object_ASSIGN_NULL(output_struct.first_obj);
    Object_ASSIGN_NULL(output_struct.second_obj);
    Object_ASSIGN_NULL(output_struct.should_be_empty);

    ASSERT(me.unimplemented(3) == Object_ERROR_INVALID);

    return Object_OK;
  }
};

extern "C" {
Object create_cpp_itest2(void) {
  ITest2Impl *me = new ITest2Impl();
  if (me == nullptr) {
    return Object_NULL;
  }
  return (Object){ImplBase::invoke, me};
}
}
