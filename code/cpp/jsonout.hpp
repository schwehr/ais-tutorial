// jsonout.hpp — a tiny, deterministic JSON writer.
//
// Why not a JSON library? Because the Python, Rust and TypeScript versions
// of this tutorial must print *byte-identical* output so that the test-suite
// can diff them, and because the tutorial has no dependencies. So we follow
// two rules everywhere:
//
//   1. Object keys are written in sorted order. `std::map` keeps its keys
//      sorted, so this happens by itself.
//   2. Every non-integer number is wrapped in `Fixed{value, decimals}` and
//      printed with exactly that many decimals (`std::format("{:.{}f}")`,
//      which rounds the exact binary value the same way Python's
//      `format(x, '.6f')`, Rust's `{:.6}` and JavaScript's `toFixed` do).
//
// Supported values: null, bool, integer, Fixed, string, array, object.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * Python has one dynamically-typed `dict`; C++ needs a type that can hold
//   "any JSON value". `std::variant<A, B, C>` (C++17) is a type-safe union:
//   it holds exactly one of the listed alternatives at a time and remembers
//   which. `std::holds_alternative<T>(v)` asks "is it a T right now?" and
//   `std::get<T>(v)` extracts it — the C++ equivalent of Python's
//   `isinstance` chain in jsonout.py.
// * The `Json` class has many constructors so that `Json(5)`, `Json("text")`,
//   `Json(true)` all work, and so that a `std::map<std::string, Json>` can be
//   built with brace initialisers: `Json::Object{{"a", 1}, {"b", "x"}}`.
//   Without the `const char*` constructor a string literal would silently
//   convert to `bool` (pointers are convertible to bool!) — a classic C++ trap.
// * `std::vector<Json>` inside `Json` is a *recursive* type. C++17 allows
//   `std::vector` of an incomplete type, which is what makes this possible.
// * `std::format` (C++20) is Python's `str.format` for C++: `{}` is a
//   placeholder, `{:.6f}` means "fixed, six decimals", and `{:.{}f}` takes
//   the precision from the next argument, like `"%.*f"` in C. It returns a
//   `std::string`, so there is no buffer to size. Both `std::format` and
//   glibc's `printf` print the correctly rounded decimal expansion of the
//   binary double, which is why the output is byte-identical to the
//   `snprintf("%.*f")` the C++17 version used.
// * `Fixed{.value = v, .decimals = d}` is a *designated initialiser*
//   (C++20): the member names are written out, so the two numbers cannot
//   be swapped by accident.

#pragma once

#include <cmath>
#include <cstdlib>
#include <format>
#include <map>
#include <optional>
#include <stdexcept>
#include <string>
#include <variant>
#include <vector>

namespace aistut {

// |value| at or above this prints as null: JSON has no nan/inf tokens, and
// above 1e21 JavaScript's toFixed switches to exponent notation, so the
// four languages could not agree on the digits anyway (see CONVENTIONS §2).
constexpr double FIXED_LIMIT = 1e21;

// A floating-point number to be printed with a fixed number of decimals.
struct Fixed {
    double value;
    int decimals;

    [[nodiscard]] std::string str() const {
        if (!std::isfinite(value) || std::fabs(value) >= FIXED_LIMIT) return "null";
        // "{:.{}f}" takes the precision from the next argument (C's "%.*f").
        std::string s = std::format("{:.{}f}", value, decimals);
        // Avoid "-0.0": if the text is negative but its numeric value is
        // zero (e.g. -0.00001 rounded to 1 decimal), drop the minus sign.
        if (s.starts_with('-') && std::strtod(s.c_str(), nullptr) == 0.0) {
            s.erase(0, 1);
        }
        return s;
    }
};

class Json {
public:
    using Array = std::vector<Json>;
    using Object = std::map<std::string, Json>;

    // --- Constructors: one per value kind we support -----------------------
    Json() : value_(nullptr) {}
    Json(std::nullptr_t) : value_(nullptr) {}
    Json(bool b) : value_(b) {}
    Json(int i) : value_(static_cast<long long>(i)) {}
    Json(long i) : value_(static_cast<long long>(i)) {}
    Json(long long i) : value_(i) {}
    Json(unsigned int i) : value_(static_cast<long long>(i)) {}
    Json(unsigned long i) : value_(static_cast<long long>(i)) {}
    Json(unsigned long long i) : value_(static_cast<long long>(i)) {}
    Json(Fixed f) : value_(f) {}
    Json(const char* s) : value_(std::string(s)) {}
    Json(std::string s) : value_(std::move(s)) {}
    Json(Array a) : value_(std::move(a)) {}
    Json(Object o) : value_(std::move(o)) {}

    // Optionals map to null when empty — Python's `None`.
    Json(const std::optional<long long>& v) : Json() { if (v) value_ = *v; }
    Json(const std::optional<int>& v) : Json() { if (v) value_ = static_cast<long long>(*v); }
    Json(const std::optional<std::string>& v) : Json() { if (v) value_ = *v; }
    Json(const std::optional<Fixed>& v) : Json() { if (v) value_ = *v; }

    // Named helpers read better than bare constructors at call sites.
    static Json fixed(double value, int decimals) { return Json(Fixed{.value = value, .decimals = decimals}); }
    static Json object() { return Json(Object{}); }
    static Json array() { return Json(Array{}); }

    // --- Inspection ---------------------------------------------------------
    [[nodiscard]] bool is_null() const { return std::holds_alternative<std::nullptr_t>(value_); }
    [[nodiscard]] bool is_object() const { return std::holds_alternative<Object>(value_); }

    // Access the object's map. Throws if this is not an object: in this
    // tutorial that is always a programming error, never a data error.
    Object& as_object() {
        if (!is_object()) throw std::logic_error("Json value is not an object");
        return std::get<Object>(value_);
    }
    const Object& as_object() const {
        if (!is_object()) throw std::logic_error("Json value is not an object");
        return std::get<Object>(value_);
    }

    // `out["key"] = value` on an object, like a Python dict.
    Json& operator[](const std::string& key) { return as_object()[key]; }

    // Python's dict.update(other): copy every key of `other` into this.
    void update(const Json& other) {
        for (const auto& [key, value] : other.as_object()) {
            as_object()[key] = value;
        }
    }

    // --- Serialisation ------------------------------------------------------
    [[nodiscard]] std::string dump() const {
        std::string out;
        write(out);
        return out;
    }

private:
    // Minimal escaping: only what JSON requires. Non-ASCII passes through as
    // UTF-8 (it never occurs in six-bit text anyway).
    static void write_string(std::string& out, const std::string& s) {
        out.push_back('"');
        for (char c : s) {
            unsigned char u = static_cast<unsigned char>(c);
            if (c == '"') {
                out += "\\\"";
            } else if (c == '\\') {
                out += "\\\\";
            } else if (c == '\n') {
                out += "\\n";
            } else if (c == '\r') {
                out += "\\r";
            } else if (c == '\t') {
                out += "\\t";
            } else if (u < 0x20) {
                out += std::format("\\u{:04x}", u);   // other control characters
            } else {
                out.push_back(c);
            }
        }
        out.push_back('"');
    }

    void write(std::string& out) const {
        if (std::holds_alternative<std::nullptr_t>(value_)) {
            out += "null";
        } else if (std::holds_alternative<bool>(value_)) {
            out += std::get<bool>(value_) ? "true" : "false";
        } else if (std::holds_alternative<long long>(value_)) {
            out += std::to_string(std::get<long long>(value_));
        } else if (std::holds_alternative<Fixed>(value_)) {
            out += std::get<Fixed>(value_).str();
        } else if (std::holds_alternative<std::string>(value_)) {
            write_string(out, std::get<std::string>(value_));
        } else if (std::holds_alternative<Array>(value_)) {
            out.push_back('[');
            bool first = true;
            for (const Json& item : std::get<Array>(value_)) {
                if (!first) out.push_back(',');
                first = false;
                item.write(out);
            }
            out.push_back(']');
        } else {
            out.push_back('{');
            bool first = true;
            // std::map iterates in key order, which gives us rule 1 for free.
            for (const auto& [key, item] : std::get<Object>(value_)) {
                if (!first) out.push_back(',');
                first = false;
                write_string(out, key);
                out.push_back(':');
                item.write(out);
            }
            out.push_back('}');
        }
    }

    std::variant<std::nullptr_t, bool, long long, Fixed, std::string, Array, Object> value_;
};

// Free function with the same name as the Python one, for familiarity.
inline std::string dumps(const Json& value) { return value.dump(); }

}  // namespace aistut
