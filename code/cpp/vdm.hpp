// vdm.hpp — the !AIVDM / !AIVDO encapsulation sentence (chapter 2).
//
//     !AIVDM,1,1,,B,15N8?>PP01JRVP6E7aLLkOvt0HAO,0*3C
//      │  │  │ │ │ │ │                            │ └ checksum
//      │  │  │ │ │ │ │                            └── fill bits (0–5)
//      │  │  │ │ │ │ └─────────────────────────────── armoured payload
//      │  │  │ │ │ └───────────────────────────────── radio channel A/B (or 1/2, or empty)
//      │  │  │ │ └─────────────────────────────────── sequential message id (0–9) for multi-sentence groups
//      │  │  │ └───────────────────────────────────── this sentence's number within the group
//      │  │  └─────────────────────────────────────── total sentences in the group
//      │  └────────────────────────────────────────── VDM = received from another station,
//      │                                                VDO = own station's transmission
//      └───────────────────────────────────────────── talker: AI mobile, AB/BS base, AN AtoN, AR receiver, …
//
// IEC 61162-1 §8.3.90 (VDM) and §8.3.91 (VDO). An AIS message longer than
// one sentence can hold (the 82-character limit leaves room for about 60
// payload characters = 360 bits) is split across several VDM sentences with
// the same sequential id; only the *last* fragment carries non-zero fill bits.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `Assembler::add` returns `std::optional<Message>`: "a complete message,
//   or nothing yet". Python returns `None` for the same case.
// * The pending groups live in a `std::map` keyed by a `std::tuple` of
//   (talker, channel, sequential id). Tuples compare element by element, so
//   they can be map keys without any extra code; `std::optional<long long>`
//   compares too (an empty optional sorts before any value).
// * `std::vector<Vdm>& group = it->second;` binds a *reference* to the vector
//   stored inside the map; `group.push_back(...)` therefore modifies the
//   map's copy, not a temporary. Dropping the `&` would be a silent bug.
// * `std::move(x)` says "I will not use `x` again, feel free to steal its
//   contents" — cheaper than copying a string or vector.

#pragma once

#include <map>
#include <optional>
#include <stdexcept>
#include <string>
#include <tuple>
#include <vector>

#include "nmea.hpp"
#include "pycompat.hpp"

namespace aistut {

struct Vdm {
    std::string talker;
    bool own_ship = false;            // true for VDO
    long long total = 0;
    long long number = 0;
    std::optional<long long> seq_id;
    std::string channel;
    std::string payload;
    long long fill_bits = 0;
    Sentence sentence;                // the enclosing NMEA sentence, for metadata
};

// Interpret the six data fields of a VDM/VDO sentence. Throws
// std::invalid_argument (Python: ValueError) with the same texts as the
// reference implementation, including Python's own int() complaint for a
// non-numeric field.
inline Vdm parse_vdm(const Sentence& s) {
    if (s.formatter != "VDM" && s.formatter != "VDO") {
        throw std::invalid_argument("not a VDM/VDO sentence: " + s.address());
    }
    if (s.fields.size() < 6) {
        throw std::invalid_argument("VDM needs 6 fields, got " + std::to_string(s.fields.size()));
    }
    Vdm v;
    v.talker = s.talker;
    v.own_ship = (s.formatter == "VDO");
    v.total = py::int_or_throw(s.fields[0]);
    v.number = py::int_or_throw(s.fields[1]);
    if (!s.fields[2].empty()) v.seq_id = py::int_or_throw(s.fields[2]);
    v.channel = s.fields[3];
    v.payload = s.fields[4];
    v.fill_bits = s.fields[5].empty() ? 0 : py::int_or_throw(s.fields[5]);
    v.sentence = s;
    return v;
}

// A complete AIS message: the concatenated payload of all its fragments.
struct Message {
    std::string payload;
    long long fill_bits = 0;
    std::string channel;
    std::string talker;
    bool own_ship = false;
    std::vector<Vdm> fragments;

    const Vdm& first() const { return fragments.front(); }
    const Vdm& last() const { return fragments.back(); }
};

// Collects VDM fragments and hands back complete messages.
//
// Fragments of a group share (talker, channel, sequential id). We keep one
// pending group per key; a fragment that does not continue the pending
// group replaces it (so a lost fragment only loses that message).
//
// Usage:
//     Assembler asm;
//     for (line : lines) {
//         std::optional<Message> msg = asm.add(parse_vdm(parse_sentence(line)));
//         if (msg) { ...decode msg->payload... }
//     }
class Assembler {
public:
    std::optional<Message> add(const Vdm& frag) {
        if (frag.total == 1) {
            return Message{frag.payload, frag.fill_bits, frag.channel,
                           frag.talker, frag.own_ship, {frag}};
        }
        Key key{frag.talker, frag.channel, frag.seq_id};
        if (frag.number == 1) {
            pending_[key] = {frag};
        } else {
            auto it = pending_.find(key);
            if (it == pending_.end() || it->second.back().number != frag.number - 1) {
                pending_.erase(key);            // out of order: drop
                return std::nullopt;
            }
            it->second.push_back(frag);
        }
        std::vector<Vdm>& group = pending_[key];
        if (frag.number == frag.total) {
            std::string payload;
            for (const Vdm& g : group) payload += g.payload;
            Message msg{std::move(payload), frag.fill_bits, frag.channel,
                        frag.talker, frag.own_ship, std::move(group)};
            pending_.erase(key);
            return msg;
        }
        return std::nullopt;
    }

private:
    using Key = std::tuple<std::string, std::string, std::optional<long long>>;
    std::map<Key, std::vector<Vdm>> pending_;
};

}  // namespace aistut
