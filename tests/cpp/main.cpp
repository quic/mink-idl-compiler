#include <object.h>
#include <stdint.h>

namespace cpp {
#include "ITest.hpp"
#include "ITest_invoke.hpp"
} // namespace cpp

#include "../c/header.h"

using cpp::ProxyBase;

class ITest1Impl : cpp::ITest1ImplBase {
public:
  ITest1Impl(struct CTest1 ctest) { this->ctest = ctest; }
  ~ITest1Impl() {}

  int32_t test_f1(uint32_t a_val, uint32_t *b_ptr) {
    return itest1_test_f1(&this->ctest, a_val, b_ptr);
  }
  int32_t in_struct(const cpp::Collection &input_ref) {
    return itest1_in_struct(&this->ctest, (const Collection *)(&input_ref));
  }
  int32_t out_struct(cpp::Collection &output_ref) {
    return itest1_out_struct(&this->ctest, (Collection *)&output_ref);
  }
  int32_t single_out(uint32_t *output_ptr) {
    return itest1_single_out(&this->ctest, output_ptr);
  }
  int32_t single_in(uint32_t input_val) {
    return itest1_single_in(&this->ctest, input_val);
  }
  int32_t single_primitive_in(const void *unused_ptr, size_t unused_len,
                              void *unused2_ptr, size_t unused2_len,
                              size_t *unused2_lenout, uint32_t input_val) {
    return itest1_single_primitive_in(&this->ctest, unused_ptr, unused_len,
                                      unused2_ptr, unused2_len, unused2_lenout,
                                      input_val);
  }
  int32_t single_primitive_out(const void *unused_ptr, size_t unused_len,
                               void *unused2_ptr, size_t unused2_len,
                               size_t *unused2_lenout, uint32_t *output_ptr) {
    return itest1_single_primitive_out(&this->ctest, unused_ptr, unused_len,
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

    CHECK_OK(itest1_multiple_primitive(
        &this->ctest, unused_ptr, unused_len, unused2_ptr, unused2_len,
        unused2_lenout, input_val, output_ptr, unused3_ref.get(), &unused4_val,
        input2_val, output2_ptr, unused5_ptr, unused5_len, unused5_lenout));

    unused4_ref = ProxyBase(unused4_val);
    return Object_OK;
  }

  int32_t
  primitive_plus_struct_in(const cpp::SingleEncapsulated &encapsulated_ref,
                           uint32_t magic_val) {
    return itest1_primitive_plus_struct_in(
        &this->ctest, (const SingleEncapsulated *)&encapsulated_ref, magic_val);
  }
  int32_t primitive_plus_struct_out(cpp::SingleEncapsulated &encapsulated_ref,
                                    uint32_t *magic_ptr) {
    return itest1_primitive_plus_struct_out(
        &this->ctest, (SingleEncapsulated *)&encapsulated_ref, magic_ptr);
  }
  int32_t bundled_with_unbundled(const cpp::SingleEncapsulated &bundled_ref,
                                 uint32_t magic_val,
                                 const cpp::Collection &unbundled_ref) {
    return itest1_bundled_with_unbundled(
        &this->ctest, (const SingleEncapsulated *)&bundled_ref, magic_val,
        (const Collection *)&unbundled_ref);
  }
  int32_t well_documented_method(uint32_t foo_val, uint32_t *bar_ptr) {
    return itest1_well_documented_method(&this->ctest, foo_val, bar_ptr);
  }

private:
  struct CTest1 ctest;
};

class ITest2Impl : cpp::ITest2ImplBase {
public:
  int32_t test_f2(const cpp::F1 &f_ref) {
    return itest2_test_f2(NULL, (const F1 *)&f_ref);
  }

  int32_t test_obj_in(const cpp::ITest1 &o_ref, uint32_t *a_ptr) {
    return itest2_test_obj_in(NULL, o_ref.get(), a_ptr);
  }
  int32_t test_obj_out(cpp::ITest1 &o_ref) {
    Object tmp = Object_NULL;
    CHECK_OK(itest2_test_obj_out(NULL, &tmp));
    o_ref = cpp::ITest1(tmp);
    return Object_OK;
  }

  int32_t test_bundle(const void *xxx_ptr, size_t xxx_len, void *yyy_ptr,
                      size_t yyy_len, size_t *yyy_lenout, uint32_t a_val,
                      const void *xxx1_ptr, size_t xxx1_len, uint8_t b_val,
                      uint32_t c_val, uint32_t *d_ptr, uint16_t *e_ptr,
                      uint32_t *f_ptr) {
    return itest2_test_bundle(NULL, xxx_ptr, xxx_len, yyy_ptr, yyy_len,
                              yyy_lenout, a_val, xxx1_ptr, xxx1_len, b_val,
                              c_val, d_ptr, e_ptr, f_ptr);
  }
  int32_t test_array(const cpp::F1 *f_in_ptr, size_t f_in_len,
                     cpp::F1 *f_out_ptr, size_t f_out_len, size_t *f_out_lenout,
                     cpp::F1 &f_x_ref, const cpp::F1 &f_y_ref,
                     const uint32_t *a_ptr, size_t a_len, uint32_t *b_ptr,
                     size_t b_len, size_t *b_lenout, int32_t *c_ptr,
                     int16_t d_val) {
    return Object_OK;
  }
};

extern "C" {

Object create_cpp_itest1(uint32_t value) {
  struct CTest1 ctest = {.refs = 1, .value = value};
  ITest1Impl *me = new ITest1Impl(ctest);
  if (me == nullptr) {
    return Object_NULL;
  }

  return (Object){cpp::ImplBase::invoke, me};
}

Object create_cpp_itest2(void) {
  ITest2Impl *me = new ITest2Impl();
  if (me == nullptr) {
    return Object_NULL;
  }
  return (Object){cpp::ImplBase::invoke, me};
}
}
