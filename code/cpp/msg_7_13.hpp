// msg_7_13.hpp — message 7 and 13: binary / safety related acknowledgement.
//
// Spec: ITU-R M.1371-5 Annex 8 §3.5 and §3.11, Table 56.
//
// STUB: this decoder has not been written yet. The chapter that covers this
// message replaces this file. Keep the signature
// `Json decode_msg_7_13(const Bits&)`.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * The parameter is deliberately unnamed (`const Bits&` with no identifier):
//   that is how C++ says "this argument exists but I do not use it" without
//   triggering an unused-parameter warning under -Wextra.
// * `throw NotImplemented(...)` plays the role of Python's
//   NotImplementedError; decode.cpp catches it and prints
//   {"msg_type":N,"not_implemented":true}.
// * `[[noreturn]]` is not used because the function will have a real return
//   value once implemented.

#pragma once

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

inline Json decode_msg_7_13(const Bits&) {
    throw NotImplemented("message 7 and 13 decoder not written yet");
}

}  // namespace aistut
