#pragma once
#include "ITest.h"
#include "object.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

const Collection TRUTH = {
    .a = 0,
    .b = 1,
    .c = 2,
    .d = 3,
};

const SingleEncapsulated TRUTH2 = {
    .inner = 0,
};

#define CHECK_OK(expr)                                                         \
  do {                                                                         \
    int32_t ret = (expr);                                                      \
    if (ret != Object_OK) {                                                    \
      printf("[%s:%d] " #expr " returned %d\n", __FILE__, __LINE__, ret);      \
      abort();                                                                 \
    }                                                                          \
  } while (0)

#define ASSERT(expr)                                                           \
  do {                                                                         \
    if (!(expr)) {                                                             \
      printf("[%s:%d] Assertion failed: " #expr "\n", __FILE__, __LINE__);     \
      abort();                                                                 \
    }                                                                          \
  } while (0)

struct CTest1 {
  uint32_t refs;
  uint32_t value;
};

#ifdef __cplusplus
extern "C" {
#endif

int32_t test_singular_object(Object itest1);

int32_t itest1_release(struct CTest1 *ctx);

int32_t itest1_retain(struct CTest1 *ctx);

int32_t itest1_test_f1(struct CTest1 *ctx, uint32_t a_val, uint32_t *b_ptr);

int32_t itest1_single_in(struct CTest1 *ctx, uint32_t input_val);

int32_t itest1_single_out(struct CTest1 *ctx, uint32_t *output_ptr);

int32_t itest1_single_primitive_in(struct CTest1 *ctx, const void *unused_ptr,
                                   size_t unused_len, void *unused2_ptr,
                                   size_t unused2_len, size_t *unused2_lenout,
                                   uint32_t input_val);
int32_t itest1_single_primitive_out(struct CTest1 *ctx, const void *unused_ptr,
                                    size_t unused_len, void *unused2_ptr,
                                    size_t unused2_len, size_t *unused2_lenout,
                                    uint32_t *output_ptr);

int32_t itest1_out_struct(struct CTest1 *ctx, Collection *output);

int32_t itest1_in_struct(struct CTest1 *ctx, const Collection *input);

int32_t itest1_out_small_struct(struct CTest1 *ctx, SingleEncapsulated *output);

int32_t itest1_in_small_struct(struct CTest1 *ctx, const SingleEncapsulated *input);

int32_t itest1_multiple_primitive(struct CTest1 *ctx, const void *unused_ptr,
                                  size_t unused_len, void *unused2_ptr,
                                  size_t unused2_len, size_t *unused2_lenout,
                                  uint16_t input_val, uint16_t *output_ptr,
                                  Object unused3_val, Object *unused4_ptr,
                                  uint32_t input2_val, uint64_t *output2_ptr,
                                  void *unused5_ptr, size_t unused5_len,
                                  size_t *unused5_lenout);

int32_t
itest1_primitive_plus_struct_in(struct CTest1 *ctx,
                                const SingleEncapsulated *encapsulated_ptr,
                                uint32_t magic_val);

int32_t itest1_primitive_plus_struct_out(struct CTest1 *ctx,
                                         SingleEncapsulated *encapsulated_ptr,
                                         uint32_t *magic_ptr);

int32_t itest1_bundled_with_unbundled(struct CTest1 *ctx,
                                      const SingleEncapsulated *bundled_ptr,
                                      uint32_t magic_val,
                                      const Collection *unbundled_ptr);

int32_t itest1_struct_array_in(struct CTest1 *ctx, const Collection *s_in_ptr, size_t s_in_len);

int32_t itest1_struct_array_out(struct CTest1 *ctx, Collection *s_out_ptr, size_t s_out_len, size_t *s_out_lenout);

int32_t itest1_well_documented_method_real(struct CTest1 *ctx, uint32_t foo_val,
                                           uint32_t *bar_ptr);

int32_t itest1_test_obj_array_in(struct CTest1 *ctx,
                                 const Object (*o_in_ptr)[3], uint32_t *a_ptr);

int32_t itest1_test_obj_array_out(struct CTest1 *ctx, Object (*o_ptr)[3],
                                  uint32_t *a_ptr);

int32_t itest1_objects_in_struct(struct CTest1 *ctx, const ObjInStruct *input,
                                 ObjInStruct *output);

#ifdef __cplusplus
}
#endif
