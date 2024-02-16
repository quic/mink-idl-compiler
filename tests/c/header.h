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

#ifdef __cplusplus
extern "C" {
#endif

int32_t itest1_release(struct CTest1 *ctx);

int32_t itest1_retain(struct CTest1 *ctx);

int32_t itest1_test_f1(struct CTest1 *ctx, uint32_t a_val, uint32_t *b_ptr);

int32_t itest1_single_in(void *ctx, uint32_t input_val);

int32_t itest1_single_out(void *ctx, uint32_t *output_ptr);

int32_t itest1_single_primitive_in(void *ctx, const void *unused_ptr,
                                   size_t unused_len, void *unused2_ptr,
                                   size_t unused2_len, size_t *unused2_lenout,
                                   uint32_t input_val);
int32_t itest1_single_primitive_out(void *ctx, const void *unused_ptr,
                                    size_t unused_len, void *unused2_ptr,
                                    size_t unused2_len, size_t *unused2_lenout,
                                    uint32_t *output_ptr);

int32_t itest1_out_struct(void *ctx, Collection *output);

int32_t itest1_in_struct(void *ctx, const Collection *input);

int32_t itest1_multiple_primitive(void *ctx, const void *unused_ptr,
                                  size_t unused_len, void *unused2_ptr,
                                  size_t unused2_len, size_t *unused2_lenout,
                                  uint16_t input_val, uint16_t *output_ptr,
                                  Object unused3_val, Object *unused4_ptr,
                                  uint32_t input2_val, uint64_t *output2_ptr,
                                  void *unused5_ptr, size_t unused5_len,
                                  size_t *unused5_lenout);

int32_t itest1_primitive_plus_struct_in(
    void *ctx, const SingleEncapsulated *encapsulated_ptr, uint32_t magic_val);

int32_t itest1_primitive_plus_struct_out(void *ctx,
                                         SingleEncapsulated *encapsulated_ptr,
                                         uint32_t *magic_ptr);

int32_t itest1_bundled_with_unbundled(void *ctx,
                                      const SingleEncapsulated *bundled_ptr,
                                      uint32_t magic_val,
                                      const Collection *unbundled_ptr);
int32_t itest1_well_documented_method(void *ctx, uint32_t foo_val,
                                      uint32_t *bar_ptr);

int32_t itest2_release(void *ctx);

int32_t itest2_retain(void *ctx);

int32_t itest2_test_f2(void *ctx, const F1 *f_ptr);

int32_t itest2_test_obj_in(void *ctx, Object o_val, uint32_t *a_ptr);

int32_t itest2_test_obj_out(void *ctx, Object *o_ptr);

int32_t itest2_test_bundle(void *ctx, const void *xxx_ptr, size_t xxx_len,
                           void *yyy_ptr, size_t yyy_len, size_t *yyy_lenout,
                           uint32_t a_val, const void *xxx1_ptr,
                           size_t xxx1_len, uint8_t b_val, uint32_t c_val,
                           uint32_t *d_ptr, uint16_t *e_ptr, uint32_t *f_ptr);

int32_t itest2_test_array(void *ctx, const F1 *f_in_ptr, size_t f_in_len,
                          F1 *f_out_ptr, size_t f_out_len, size_t *f_out_lenout,
                          F1 *f_x_ptr, const F1 *f_y_ptr, const uint32_t *a_ptr,
                          size_t a_len, uint32_t *b_ptr, size_t b_len,
                          size_t *b_lenout, int32_t *c_ptr, int16_t d_val);

int32_t itest2_test_obj_array_in(void *ctx, const Object (*o_in_ptr)[3], uint32_t *a_ptr);

int32_t itest2_test_obj_array_out(void *ctx, Object (*o_out_ptr)[3], uint32_t *a_ptr);

#ifdef __cplusplus
}
#endif
