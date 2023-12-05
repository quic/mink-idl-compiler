#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ITest.h"
#include "ITest_invoke.h"
#include "object.h"

const Collection TRUTH = {
    .a = 0,
    .b = 1,
    .c = 2,
    .d = 3,
};

#define CHECK_OK(expr)                                                         \
  do {                                                                         \
    int32_t ret = (expr);                                                      \
    if (ret != Object_OK) {                                                    \
      printf(#expr " returned %d\n", ret);                                     \
      return ret;                                                              \
    }                                                                          \
  } while (0)

#define ASSERT(expr)                                                           \
  do {                                                                         \
    if (!(expr)) {                                                             \
      printf("Assertion failed: " #expr "\n");                                 \
      return Object_ERROR;                                                     \
    }                                                                          \
  } while (0)

struct CTest1 {
  uint32_t refs;
  uint32_t value;
};

static inline int32_t itest1_release(struct CTest1 *ctx) {
  if (--ctx->refs == 0) {
    free(ctx);
  }
  return Object_OK;
}

static inline int32_t itest1_retain(struct CTest1 *ctx) {
  ctx->refs++;
  return Object_OK;
}

static inline int32_t itest1_test_f1(struct CTest1 *ctx, uint32_t a_val,
                                     uint32_t *b_ptr) {
  *b_ptr = a_val + ctx->value;
  return Object_OK;
}

int32_t itest1_single_in(void *ctx, uint32_t input_val) {
  if (input_val == 0xdead) {
    return Object_OK;
  } else {
    return Object_ERROR;
  }
}

int32_t itest1_single_out(void *ctx, uint32_t *output_ptr) {
  *output_ptr = 0xdead;
  return Object_OK;
}

int32_t itest1_single_primitive_in(void *ctx, const void *unused_ptr,
                                   size_t unused_len, void *unused2_ptr,
                                   size_t unused2_len, size_t *unused2_lenout,
                                   uint32_t input_val) {
  return itest1_single_in(ctx, input_val);
}
int32_t itest1_single_primitive_out(void *ctx, const void *unused_ptr,
                                    size_t unused_len, void *unused2_ptr,
                                    size_t unused2_len, size_t *unused2_lenout,
                                    uint32_t *output_ptr) {
  return itest1_single_out(ctx, output_ptr);
}

int32_t itest1_out_struct(void *ctx, Collection *output) {
  memcpy(output, &TRUTH, sizeof(TRUTH));
  return Object_OK;
}

int32_t itest1_in_struct(void *ctx, const Collection *input) {
  ASSERT(memcmp(input, &TRUTH, sizeof(TRUTH)) == 0);
  return Object_OK;
}

int32_t itest1_multiple_primitive(void *ctx, const void *unused_ptr,
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

int32_t itest1_primitive_plus_struct_in(
    void *ctx, const SingleEncapsulated *encapsulated_ptr, uint32_t magic_val) {
  ASSERT(encapsulated_ptr->inner == SUCCESS_FLAG && magic_val == SUCCESS_FLAG);
  return Object_OK;
}

int32_t itest1_primitive_plus_struct_out(void *ctx,
                                         SingleEncapsulated *encapsulated_ptr,
                                         uint32_t *magic_ptr) {
  encapsulated_ptr->inner = SUCCESS_FLAG;
  *magic_ptr = SUCCESS_FLAG;
  return Object_OK;
}

int32_t itest1_bundled_with_unbundled(void *ctx,
                                      const SingleEncapsulated *bundled_ptr,
                                      uint32_t magic_val,
                                      const Collection *unbundled_ptr) {
  ASSERT(bundled_ptr->inner == SUCCESS_FLAG);
  ASSERT(magic_val == SUCCESS_FLAG);
  ASSERT(memcmp(unbundled_ptr, &TRUTH, sizeof(TRUTH)) == 0);

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

int32_t itest2_test_f2(void *ctx, const F1 *f_ptr) { return Object_OK; }

int32_t itest2_test_obj_in(void *ctx, Object o_val, uint32_t *a_ptr) {
  if (Object_isNull(o_val)) {
    return Object_ERROR_BADOBJ;
  }
  uint8_t empty[] = {};
  size_t lenout = 0;
  Object empty_o = Object_NULL;
  uint16_t flag1 = 0;
  uint64_t flag2 = 0;
  const SingleEncapsulated single_encapsulated = {.inner = SUCCESS_FLAG};

  CHECK_OK(ITest1_single_in(o_val, SUCCESS_FLAG));
  CHECK_OK(ITest1_single_primitive_in(o_val, empty, sizeof(empty), empty,
                                      sizeof(empty), &lenout, SUCCESS_FLAG));
  CHECK_OK(ITest1_primitive_plus_struct_in(o_val, &single_encapsulated,
                                           SUCCESS_FLAG));
  CHECK_OK(ITest1_multiple_primitive(
      o_val, empty, sizeof(empty), empty, sizeof(empty), &lenout, SUCCESS_FLAG,
      &flag1, Object_NULL, &empty_o, SUCCESS_FLAG, &flag2, empty, sizeof(empty),
      &lenout));
  ASSERT(flag1 == SUCCESS_FLAG);
  ASSERT(flag2 == SUCCESS_FLAG);
  CHECK_OK(ITest1_bundled_with_unbundled(o_val, &single_encapsulated,
                                         SUCCESS_FLAG, &TRUTH));
  {
    uint32_t out = 0;
    CHECK_OK(ITest1_single_out(o_val, &out));
    ASSERT(out == SUCCESS_FLAG);
  }
  {
    uint32_t out = 0;
    CHECK_OK(ITest1_single_primitive_out(o_val, empty, sizeof(empty), empty,
                                         sizeof(empty), &lenout, &out));
    ASSERT(out == SUCCESS_FLAG);
  }
  {
    SingleEncapsulated single_encapsulated = {0};
    uint32_t out = 0;
    CHECK_OK(
        ITest1_primitive_plus_struct_out(o_val, &single_encapsulated, &out));
    ASSERT(out == SUCCESS_FLAG);
    ASSERT(single_encapsulated.inner == SUCCESS_FLAG);
  }

  *a_ptr = SUCCESS_FLAG;
  return Object_OK;
}

int32_t itest2_test_obj_out(void *ctx, Object *o_ptr) {
  *o_ptr = create_c_itest1(42);
  return Object_OK;
}

int32_t itest2_test_bundle(void *ctx, const void *xxx_ptr, size_t xxx_len,
                           void *yyy_ptr, size_t yyy_len, size_t *yyy_lenout,
                           uint32_t a_val, const void *xxx1_ptr,
                           size_t xxx1_len, uint8_t b_val, uint32_t c_val,
                           uint32_t *d_ptr, uint16_t *e_ptr, uint32_t *f_ptr) {
  return Object_ERROR;
}

int32_t itest2_test_array(void *ctx, const F1 *f_in_ptr, size_t f_in_len,
                          F1 *f_out_ptr, size_t f_out_len, size_t *f_out_lenout,
                          F1 *f_x_ptr, const F1 *f_y_ptr, const uint32_t *a_ptr,
                          size_t a_len, uint32_t *b_ptr, size_t b_len,
                          size_t *b_lenout, int32_t *c_ptr, int16_t d_val) {
  return Object_ERROR;
}

ITest2_DEFINE_INVOKE(itest2_invoke, itest2_, void *);

Object create_c_itest2() { return (Object){itest2_invoke, NULL}; }
