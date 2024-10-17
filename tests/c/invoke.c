// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

#include "header.h"

#include <string.h>

#include "ITest_invoke.h"
#include "object.h"

Object create_c_itest1(uint32_t value);

int32_t test_singular_object(Object itest1) {
  if (Object_isNull(itest1)) {
    return Object_ERROR_BADOBJ;
  }
  uint8_t empty[] = {};
  size_t lenout = 0;
  Object empty_o = Object_NULL;
  uint16_t flag1 = 0;
  uint64_t flag2 = 0;
  const SingleEncapsulated single_encapsulated = {.inner = SUCCESS_FLAG};

  CHECK_OK(ITest1_single_in(itest1, SUCCESS_FLAG));
  CHECK_OK(ITest1_single_primitive_in(itest1, empty, sizeof(empty), empty,
                                      sizeof(empty), &lenout, SUCCESS_FLAG));
  CHECK_OK(ITest1_primitive_plus_struct_in(itest1, &single_encapsulated,
                                           SUCCESS_FLAG));
  CHECK_OK(ITest1_multiple_primitive(
      itest1, empty, sizeof(empty), empty, sizeof(empty), &lenout, SUCCESS_FLAG,
      &flag1, Object_NULL, &empty_o, SUCCESS_FLAG, &flag2, empty, sizeof(empty),
      &lenout));
  ASSERT(flag1 == SUCCESS_FLAG);
  ASSERT(flag2 == SUCCESS_FLAG);
  CHECK_OK(ITest1_bundled_with_unbundled(itest1, &single_encapsulated,
                                         SUCCESS_FLAG, &TRUTH));
  {
    uint32_t out = 0;
    CHECK_OK(ITest1_single_out(itest1, &out));
    ASSERT(out == SUCCESS_FLAG);
  }
  {
    uint32_t out = 0;
    CHECK_OK(ITest1_single_primitive_out(itest1, empty, sizeof(empty), empty,
                                         sizeof(empty), &lenout, &out));
    ASSERT(out == SUCCESS_FLAG);
  }
  {
    SingleEncapsulated single_encapsulated = {0};
    uint32_t out = 0;
    CHECK_OK(
        ITest1_primitive_plus_struct_out(itest1, &single_encapsulated, &out));
    ASSERT(out == SUCCESS_FLAG);
    ASSERT(single_encapsulated.inner == SUCCESS_FLAG);
  }
  {
    uint32_t out = 0;
    CHECK_OK(ITest1_well_documented_method(itest1, SUCCESS_FLAG, &out));
    ASSERT(out == SUCCESS_FLAG);
  }

  return Object_OK;
}

int32_t itest1_release(struct CTest1 *ctx) {
  if (--ctx->refs == 0) {
    free(ctx);
  }
  return Object_OK;
}

int32_t itest1_retain(struct CTest1 *ctx) {
  ctx->refs++;
  return Object_OK;
}

int32_t itest1_test_f1(struct CTest1 *ctx, uint32_t a_val, uint32_t *b_ptr) {
  *b_ptr = a_val + ctx->value;
  return Object_OK;
}

int32_t itest1_single_in(struct CTest1 *ctx, uint32_t input_val) {
  if (input_val == 0xdead) {
    return Object_OK;
  } else {
    return Object_ERROR;
  }
}

int32_t itest1_single_out(struct CTest1 *ctx, uint32_t *output_ptr) {
  *output_ptr = 0xdead;
  return Object_OK;
}

int32_t itest1_single_primitive_in(struct CTest1 *ctx, const void *unused_ptr,
                                   size_t unused_len, void *unused2_ptr,
                                   size_t unused2_len, size_t *unused2_lenout,
                                   uint32_t input_val) {
  return itest1_single_in(ctx, input_val);
}
int32_t itest1_single_primitive_out(struct CTest1 *ctx, const void *unused_ptr,
                                    size_t unused_len, void *unused2_ptr,
                                    size_t unused2_len, size_t *unused2_lenout,
                                    uint32_t *output_ptr) {
  return itest1_single_out(ctx, output_ptr);
}

int32_t itest1_out_struct(struct CTest1 *ctx, Collection *output) {
  memcpy(output, &TRUTH, sizeof(TRUTH));
  return Object_OK;
}

int32_t itest1_in_struct(struct CTest1 *ctx, const Collection *input) {
  ASSERT(memcmp(input, &TRUTH, sizeof(TRUTH)) == 0);
  return Object_OK;
}

int32_t itest1_out_small_struct(struct CTest1 *ctx, SingleEncapsulated *output) {
  memcpy(output, &TRUTH2, sizeof(TRUTH2));
  return Object_OK;
}

int32_t itest1_in_small_struct(struct CTest1 *ctx, const SingleEncapsulated *input) {
  ASSERT(memcmp(input, &TRUTH2, sizeof(TRUTH2)) == 0);
  return Object_OK;
}

int32_t itest1_multiple_primitive(struct CTest1 *ctx, const void *unused_ptr,
                                  size_t unused_len, void *unused2_ptr,
                                  size_t unused2_len, size_t *unused2_lenout,
                                  uint16_t input_val, uint16_t *output_ptr,
                                  Object unused3_val, Object *unused4_ptr,
                                  uint32_t input2_val, uint64_t *output2_ptr,
                                  void *unused5_ptr, size_t unused5_len,
                                  size_t *unused5_lenout) {
  if (input_val != SUCCESS_FLAG || input2_val != SUCCESS_FLAG) {
    return Object_ERROR;
  }
  *output_ptr = SUCCESS_FLAG;
  *output2_ptr = SUCCESS_FLAG;
  return Object_OK;
}

int32_t
itest1_primitive_plus_struct_in(struct CTest1 *ctx,
                                const SingleEncapsulated *encapsulated_ptr,
                                uint32_t magic_val) {
  ASSERT(encapsulated_ptr->inner == SUCCESS_FLAG && magic_val == SUCCESS_FLAG);
  return Object_OK;
}

int32_t itest1_primitive_plus_struct_out(struct CTest1 *ctx,
                                         SingleEncapsulated *encapsulated_ptr,
                                         uint32_t *magic_ptr) {
  encapsulated_ptr->inner = SUCCESS_FLAG;
  *magic_ptr = SUCCESS_FLAG;
  return Object_OK;
}

int32_t itest1_primitive_array_in_struct(struct CTest1 *ctx,
                                         ArrInStruct *arr_ptr,
                                         uint32_t *magic_ptr) {
  arr_ptr->a[0] = 7;
  arr_ptr->a[1] = 8;
  arr_ptr->c[0].a = 9;
  arr_ptr->c[0].b = 7;
  arr_ptr->c[1].a = 8;
  arr_ptr->c[1].b = 9;
  arr_ptr->d = SUCCESS_FLAG;
  *magic_ptr = SUCCESS_FLAG;
  return Object_OK;
}

int32_t itest1_bundled_with_unbundled(struct CTest1 *ctx,
                                      const SingleEncapsulated *bundled_ptr,
                                      uint32_t magic_val,
                                      const Collection *unbundled_ptr) {
  ASSERT(bundled_ptr->inner == SUCCESS_FLAG);
  ASSERT(magic_val == SUCCESS_FLAG);
  ASSERT(memcmp(unbundled_ptr, &TRUTH, sizeof(TRUTH)) == 0);

  return Object_OK;
}

int32_t itest1_struct_array_in(struct CTest1 *ctx, const Collection *s_in_ptr, size_t s_in_len) {
  for (size_t i = 0; i < s_in_len; i++) {
    ASSERT(memcmp(&s_in_ptr[i], &TRUTH, sizeof(TRUTH)) == 0);
  }
  return Object_OK;
}

int32_t itest1_struct_array_out(struct CTest1 *ctx, Collection *s_out_ptr, size_t s_out_len, size_t *s_out_lenout) {
  for (size_t i = 0; i < s_out_len; i++) {
    memcpy(&s_out_ptr[i], &TRUTH, sizeof(TRUTH));
  }
  *s_out_lenout = s_out_len;
  return Object_OK;
}

int32_t itest1_well_documented_method_real(struct CTest1 *ctx, uint32_t foo_val,
                                           uint32_t *bar_ptr) {
  ASSERT(foo_val == SUCCESS_FLAG);
  *bar_ptr = SUCCESS_FLAG;
  return Object_OK;
}

#define itest1_well_documented_method(a, b, c)                                 \
  itest1_well_documented_method_real(a, b, c)

int32_t itest1_test_obj_array_in(struct CTest1 *ctx,
                                 const Object (*o_in_ptr)[3], uint32_t *a_ptr) {
  for (size_t i = 0; i < 3; i++) {
    Object o = (*o_in_ptr)[i];
    if (!Object_isNull(o)) {
      CHECK_OK(test_singular_object(o));
    }
  }
  *a_ptr = SUCCESS_FLAG;
  return Object_OK;
}

int32_t itest1_test_obj_array_out(struct CTest1 *ctx, Object (*o_ptr)[3],
                                  uint32_t *a_ptr) {
  (*o_ptr)[0] = create_c_itest1(0);
  (*o_ptr)[1] = create_c_itest1(1);
  (*o_ptr)[2] = create_c_itest1(2);
  *a_ptr = SUCCESS_FLAG;
  return Object_OK;
}

int32_t itest1_objects_in_struct(struct CTest1 *ctx, const ObjInStruct *input,
                                 ObjInStruct *output) {
  CHECK_OK(test_singular_object(input->first_obj));
  ASSERT(Object_isNull(input->should_be_empty));
  CHECK_OK(test_singular_object(input->second_obj));

  for (size_t i = 0; i < sizeof(input->p1) / sizeof(input->p1[0]); i++) {
    ASSERT(input->p1[i] == SUCCESS_FLAG);
    ASSERT(input->p2[i] == SUCCESS_FLAG);
    ASSERT(input->p3[i] == SUCCESS_FLAG);

    output->p1[i] = SUCCESS_FLAG;
    output->p2[i] = SUCCESS_FLAG;
    output->p3[i] = SUCCESS_FLAG;
  }
  output->first_obj = create_c_itest1(1);
  output->second_obj = create_c_itest1(2);
  output->should_be_empty = Object_NULL;

  return Object_OK;
}

ITest1_DEFINE_INVOKE(itest1_invoke, itest1_, struct CTest1 *);

Object create_c_itest1(uint32_t value) {
  struct CTest1 *ctx = (struct CTest1 *)malloc(sizeof(struct CTest1));
  if (!ctx)
    return Object_NULL;

  ctx->refs = 1;
  ctx->value = value;
  return (Object){itest1_invoke, ctx};
}

int32_t itest2_release(void *ctx) { return Object_OK; }

int32_t itest2_retain(void *ctx) { return Object_OK; }

int32_t itest2_entrypoint(void *ctx, Object itest1) {
  ASSERT(!Object_isNull(itest1));
  CHECK_OK(test_singular_object(itest1));

  Object objects[3] = {create_c_itest1(1), Object_NULL, create_c_itest1(2)};
  Object objects_out[3] = {0};
  uint32_t a = 0;
  CHECK_OK(ITest1_test_obj_array_in(itest1, &objects, &a));
  ASSERT(a == SUCCESS_FLAG);
  a = 0;
  CHECK_OK(ITest1_test_obj_array_out(itest1, &objects_out, &a));
  ASSERT(a == SUCCESS_FLAG);

  for (size_t i = 0; i < sizeof(objects) / sizeof(objects[0]); i++) {
    Object_ASSIGN_NULL(objects[i]);

    CHECK_OK(test_singular_object(objects_out[i]));
    Object_ASSIGN_NULL(objects_out[i]);
  }

  const uint32_t VALID_PS[4] = {SUCCESS_FLAG, SUCCESS_FLAG, SUCCESS_FLAG,
                                SUCCESS_FLAG};

  ObjInStruct input_struct = {
      .first_obj = create_c_itest1(1),
      .should_be_empty = Object_NULL,
      .second_obj = create_c_itest1(2),
  };
  memcpy(&input_struct.p1, VALID_PS, sizeof(VALID_PS));
  memcpy(&input_struct.p2, VALID_PS, sizeof(VALID_PS));
  memcpy(&input_struct.p3, VALID_PS, sizeof(VALID_PS));
  ObjInStruct output_struct = {0};
  CHECK_OK(ITest1_objects_in_struct(itest1, &input_struct, &output_struct));
  ASSERT(memcmp(&output_struct.p1, VALID_PS, sizeof(VALID_PS)) == 0);
  ASSERT(memcmp(&output_struct.p2, VALID_PS, sizeof(VALID_PS)) == 0);
  ASSERT(memcmp(&output_struct.p3, VALID_PS, sizeof(VALID_PS)) == 0);
  CHECK_OK(test_singular_object(output_struct.first_obj));
  CHECK_OK(test_singular_object(output_struct.second_obj));
  ASSERT(Object_isNull(output_struct.should_be_empty));

  Object_ASSIGN_NULL(input_struct.first_obj);
  Object_ASSIGN_NULL(input_struct.second_obj);
  Object_ASSIGN_NULL(input_struct.should_be_empty);

  Object_ASSIGN_NULL(output_struct.first_obj);
  Object_ASSIGN_NULL(output_struct.second_obj);
  Object_ASSIGN_NULL(output_struct.should_be_empty);

  ASSERT(ITest1_unimplemented(itest1, 3) == Object_ERROR_INVALID);

  return Object_OK;
}

ITest2_DEFINE_INVOKE(itest2_invoke, itest2_, void *);

Object create_c_itest2() { return (Object){itest2_invoke, NULL}; }
