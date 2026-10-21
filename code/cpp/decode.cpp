// decode.cpp — command-line driver: NMEA/AIS log lines in, JSON lines out.
//
//     build/decode FILE [FILE...]        # or '-' for standard input
//
// For every input line we print at most one JSON object (see jsonout.hpp for
// why the JSON is hand-rolled). The same program exists in Python, Rust and
// TypeScript, and the test-suite checks that all four print identical text.
//
// Output shapes
// -------------
// GPS sentence          {"line":7,"sentence":"GGA", ...fields...}
// AIS message           {"line":9,"ais":{...decoded...},"channel":"A","talker":"AI",
//                        "own_ship":false,"payload_bits":168,"fragments":1,
//                        "uscg":{...},"tagblock":{...}}
//                       ("uscg"/"tagblock" only when present)
// unknown formatter     {"line":3,"address":"PMTK010","skipped":"no parser"}
// problem               {"line":5,"error":"checksum mismatch","raw":"..."}
// Logger comments (# ...) and blank lines print nothing.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * This is the only .cpp file; everything else is a header. The compiler
//   therefore sees the whole program at once ("one translation unit"),
//   which is why the Makefile has a single compile command.
// * `int main(int argc, char** argv)`: `argc` is the number of command-line
//   words, `argv` the words themselves; `argv[0]` is the program name, so
//   real arguments start at index 1.
// * Input is read as raw bytes (`std::ios::binary`) and split into lines the
//   way Python's text mode does ("universal newlines": "\n", "\r\n" and a
//   lone "\r" all end a line), then passed through a UTF-8 repair step. That
//   keeps the C++ output identical to Python's even on damaged log files.
// * `try { ... } catch (const SomeError& e) { ... }` is C++ exception
//   handling. Catching by `const&` avoids copying the exception object and
//   preserves its dynamic type. The order of catch clauses matters: more
//   specific types first.
// * `auto [tag, rest] = ...` is a structured binding (C++17) that unpacks a
//   pair into two named variables.
// * RAII: the `std::ifstream` closes its file automatically when it goes
//   out of scope — no `with` block or explicit close() needed.

#include <fstream>
#include <iostream>
#include <istream>
#include <map>
#include <optional>
#include <stdexcept>
#include <string>
#include <vector>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"
#include "nmea.hpp"
#include "pycompat.hpp"
#include "tagblock.hpp"
#include "uscg.hpp"
#include "vdm.hpp"

// One header per message type (or family). Each exposes decode_msg_N(bits) -> Json.
#include "msg_1_2_3.hpp"
#include "msg_4_11.hpp"
#include "msg_5.hpp"
#include "msg_6.hpp"
#include "msg_7_13.hpp"
#include "msg_8.hpp"
#include "msg_9.hpp"
#include "msg_10.hpp"
#include "msg_12.hpp"
#include "msg_14.hpp"
#include "msg_15.hpp"
#include "msg_16.hpp"
#include "msg_17.hpp"
#include "msg_18.hpp"
#include "msg_19.hpp"
#include "msg_20.hpp"
#include "msg_21.hpp"
#include "msg_22.hpp"
#include "msg_23.hpp"
#include "msg_24.hpp"
#include "msg_25.hpp"
#include "msg_26.hpp"
#include "msg_27.hpp"

namespace aistut {

// A pointer to a function taking `const Bits&` and returning `Json`.
using Decoder = Json (*)(const Bits&);

const std::map<int, Decoder> DECODERS = {
    {1, decode_msg_1_2_3}, {2, decode_msg_1_2_3}, {3, decode_msg_1_2_3},
    {4, decode_msg_4_11}, {11, decode_msg_4_11},
    {5, decode_msg_5},
    {6, decode_msg_6},
    {7, decode_msg_7_13}, {13, decode_msg_7_13},
    {8, decode_msg_8},
    {9, decode_msg_9},
    {10, decode_msg_10},
    {12, decode_msg_12},
    {14, decode_msg_14},
    {15, decode_msg_15},
    {16, decode_msg_16},
    {17, decode_msg_17},
    {18, decode_msg_18},
    {19, decode_msg_19},
    {20, decode_msg_20},
    {21, decode_msg_21},
    {22, decode_msg_22},
    {23, decode_msg_23},
    {24, decode_msg_24},
    {25, decode_msg_25},
    {26, decode_msg_26},
    {27, decode_msg_27},
};

// Turn an assembled VDM payload into a JSON object (or an error entry).
Json decode_ais(const Message& msg) {
    std::optional<Bits> bits;
    try {
        bits = Bits::from_payload(msg.payload, msg.fill_bits);
    } catch (const std::invalid_argument& e) {
        return Json::Object{{"error", e.what()}};
    }
    if (bits->size() < 6) {
        return Json::Object{
            {"error", "payload too short: " + std::to_string(bits->size()) + " bits"},
            {"payload_bits", bits->size()},
        };
    }
    int msg_type = static_cast<int>(bits->uint(0, 6));
    auto found = DECODERS.find(msg_type);
    Json out = Json::Object{{"payload_bits", bits->size()}};
    if (found == DECODERS.end()) {
        out["error"] = "unknown message type " + std::to_string(msg_type);
        out["msg_type"] = msg_type;
        return out;
    }
    Decoder decoder = found->second;
    try {
        out["ais"] = decoder(*bits);
    } catch (const DecodeError& e) {
        out["error"] = e.what();
        out["msg_type"] = msg_type;
    } catch (const NotImplemented&) {
        out["ais"] = Json::Object{{"msg_type", msg_type}, {"not_implemented", true}};
    }
    return out;
}

// Handle one text line. Returns a JSON object to print, or nothing to stay quiet.
std::optional<Json> process_line(const std::string& line, long long lineno, Assembler& asm_) {
    std::string stripped = py::strip(line);
    if (stripped.empty() || stripped[0] == '#') return std::nullopt;
    Json out = Json::Object{{"line", lineno}};

    std::optional<TagBlock> tag;
    std::string rest;
    try {
        auto [found_tag, remainder] = split_tag_block(stripped);
        tag = found_tag;
        rest = remainder;
    } catch (const std::invalid_argument& e) {
        out["error"] = e.what();
        out["raw"] = stripped;
        return out;
    }
    if (tag) out["tagblock"] = tag->to_dict();

    Sentence s;
    try {
        s = parse_sentence(rest);
    } catch (const NmeaError& e) {
        out["error"] = e.what();
        out["raw"] = stripped;
        return out;
    }
    if (!s.after_checksum.empty()) out["uscg"] = parse_uscg_fields(s.after_checksum);
    if (!s.checksum_ok) {
        out["error"] = s.checksum_given ? "checksum mismatch" : "no checksum";
        out["raw"] = s.raw;
        return out;
    }

    if (s.formatter == "VDM" || s.formatter == "VDO") {
        Vdm frag;
        try {
            frag = parse_vdm(s);
        } catch (const std::invalid_argument& e) {
            out["error"] = e.what();
            out["raw"] = s.raw;
            return out;
        }
        std::optional<Message> msg = asm_.add(frag);
        if (!msg) return std::nullopt;             // waiting for more fragments
        out.update(decode_ais(*msg));
        out["channel"] = msg->channel;
        out["talker"] = msg->talker;
        out["own_ship"] = msg->own_ship;
        out["fragments"] = msg->fragments.size();
        return out;
    }

    GpsParser parser = nullptr;
    if (s.talker != "P") {
        auto it = GPS_PARSERS.find(s.formatter);
        if (it != GPS_PARSERS.end()) parser = it->second;
    }
    if (parser == nullptr) {
        out["address"] = s.address();
        out["skipped"] = "no parser";
        return out;
    }
    // A GPS sentence with a valid checksum can still carry a non-numeric
    // field. The py:: helpers throw std::invalid_argument with Python's
    // exact ValueError text; report it like any other bad line.
    try {
        out.update(parser(s));
    } catch (const std::invalid_argument& e) {
        out["error"] = e.what();
        out["raw"] = s.raw;
    }
    return out;
}

// Read one line with Python's "universal newlines" rule. Returns false at
// end of input (a final line without a terminator is still returned once).
bool read_line(std::istream& in, std::string& line) {
    line.clear();
    bool any = false;
    int c;
    while ((c = in.get()) != std::char_traits<char>::eof()) {
        any = true;
        if (c == '\n') return true;
        if (c == '\r') {
            if (in.peek() == '\n') in.get();       // "\r\n" is a single terminator
            return true;
        }
        line.push_back(static_cast<char>(c));
    }
    return any;
}

int run(std::istream& in, long long& lineno, Assembler& asm_) {
    std::string raw_line;
    while (read_line(in, raw_line)) {
        ++lineno;
        std::string line = py::utf8_replace_invalid(raw_line);
        std::optional<Json> result = process_line(line, lineno, asm_);
        if (result) std::cout << result->dump() << '\n';
    }
    return 0;
}

}  // namespace aistut

int main(int argc, char** argv) {
    if (argc < 2) {
        std::cout << "usage: decode FILE [FILE...]   (use '-' for standard input)\n"
                     "Prints one JSON object per NMEA/AIS line; see decode.cpp for the shapes.\n";
        return 2;
    }
    // Do not synchronise C++ streams with C stdio: much faster output.
    std::ios::sync_with_stdio(false);

    aistut::Assembler assembler;
    long long lineno = 0;
    try {
        for (int i = 1; i < argc; ++i) {
            std::string name = argv[i];
            if (name == "-") {
                aistut::run(std::cin, lineno, assembler);
                continue;
            }
            std::ifstream file(name, std::ios::binary);
            if (!file) {
                std::cerr << "decode: cannot open " << name << '\n';
                return 1;
            }
            aistut::run(file, lineno, assembler);
        }   // `file` is closed automatically here (RAII)
    } catch (const std::exception& e) {
        // Data problems are all reported as JSON error objects above; this
        // only catches programming errors (and keeps the output written so
        // far). Flush, say what happened, exit non-zero like Python would.
        std::cout.flush();
        std::cerr << "decode: line " << lineno << ": " << e.what() << '\n';
        return 1;
    }
    std::cout.flush();
    return 0;
}
