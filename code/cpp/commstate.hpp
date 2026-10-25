// commstate.hpp — TDMA communication state schedule and timing helpers (chapter 17).
//
// Spec: ITU-R M.1371-5 Annex 2 §3.1 (synchronisation and frame structure),
// §3.3.4 (SOTDMA and ITDMA slot allocation), §3.3.7.2.2 Tables 18/19 (SOTDMA
// communication state), and §3.3.7.3.2 Table 20 (ITDMA communication state).
//
// Eight AIS message types end with a communication state:
//
//     1, 2      168 bits   bits 149-167 (19)   always SOTDMA
//     3         168 bits   bits 149-167 (19)   always ITDMA
//     4, 11     168 bits   bits 149-167 (19)   always SOTDMA
//     9         168 bits   bits 148-167 (20)   selector at 148 + 19-bit state
//     18        168 bits   bits 148-167 (20)   selector at 148 + 19-bit state
//     26     <=1064 bits   last 20 bits        selector + 19-bit state
//
// `msg_1_2_3.hpp` decodes the 19-bit SOTDMA and ITDMA bit fields into a Json
// object; this header pulls in those decoders and adds the frame-arithmetic
// and clock-recovery functions that turn raw slot numbers, offsets, and UTC
// sub-messages into physical schedules and UTC arrival times.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `std::optional<int>` is C++'s `int | None`. Returning `std::nullopt`
//   makes "slot de-allocated after transmission" (`slot_offset == 0`) or an
//   out-of-range slot number a distinct value that the caller cannot
//   accidentally add to a slot index.
// * In C++, `-5 % 60` is `-5` rather than `55`; `((whole % N) + N) % N`
//   reproduces Python's floor-division remainder so that `day_start` and
//   `minute_start` round down to a multiple of `N` for any integer.

#pragma once

#include <cmath>
#include <optional>

#include "jsonout.hpp"
#include "msg_1_2_3.hpp"

namespace aistut {

// Annex 2 §3.2.1 Table 10: one 60 s frame has 2 250 slots (26.667 ms each).
inline constexpr int SLOTS_PER_FRAME = 2250;
inline constexpr double FRAME_DURATION_S = 60.0;
inline constexpr double SLOT_DURATION_S = FRAME_DURATION_S / SLOTS_PER_FRAME;

// Annex 2 §3.3.4.4 Table 16: a newly allocated SOTDMA slot is assigned a
// random integer time-out between TMO_MIN (3) and TMO_MAX (7) frames.
inline constexpr int TMO_MIN = 3;
inline constexpr int TMO_MAX = 7;

// Convert a slot number (`0..2249`) to seconds into the UTC minute.
//
// Each TDMA frame is 60 s divided into 2 250 slots, so slot `k` begins at
// `k * 60 / 2250` seconds past the minute [Annex 2 §3.2.1]. Returns null
// if `slot_number` is outside `0..2249`.
inline Json slot_to_seconds(int slot_number) {
    if (slot_number < 0 || slot_number >= SLOTS_PER_FRAME) return Json();
    return Json::fixed(slot_number * FRAME_DURATION_S / SLOTS_PER_FRAME, 3);
}

// Return the slot in the *next* frame announced by a SOTDMA `slot_offset`.
//
// When a SOTDMA stream reaches `slot_timeout == 0`, its 14-bit sub-message
// is a `slot_offset` relative to the current slot on the same channel
// [Annex 2 §3.3.7.2.2 Table 19]:
//   * `slot_offset == 0` means the slot is de-allocated after this
//     transmission (`std::nullopt`);
//   * any non-zero offset selects `(slot_number + slot_offset) % 2250` in
//     the next frame.
inline std::optional<int> next_sotdma_slot(int slot_number, int slot_offset) {
    if (slot_number < 0 || slot_number >= SLOTS_PER_FRAME || slot_offset <= 0) {
        return std::nullopt;
    }
    return (slot_number + slot_offset) % SLOTS_PER_FRAME;
}

// Return the slot of the next transmission announced by an ITDMA `slot_offset`.
//
// In ITDMA (`decode_itdma`), `slot_offset` is already expanded to
// `slot_increment` (`0..8191`) or `slot_increment + 8192` when `slots_raw`
// is 5, 6, or 7 [Annex 2 §3.3.7.3.2 Table 20]. An offset of `0` means this
// is the last transmission for the slot (`std::nullopt`); otherwise the next
// transmission on the same channel lands in `(slot_number + slot_offset) % 2250`.
inline std::optional<int> next_itdma_slot(int slot_number, int slot_offset) {
    if (slot_number < 0 || slot_number >= SLOTS_PER_FRAME || slot_offset <= 0) {
        return std::nullopt;
    }
    return (slot_number + slot_offset) % SLOTS_PER_FRAME;
}

// Recover the UTC transmission timestamp from a logger clock and SOTDMA state.
//
// When `utc_hour` (`0..23`) and `utc_minute` (`0..59`) are known — from a
// nearby `slot_timeout == 1` sub-message — the full time-of-day within the
// 86 400 s UTC day is fixed, resolving logger clock offsets of many minutes
// or hours. Otherwise we fall back to the nearest minute within 30 s of
// `unix_time` (the same rule as `time_of_arrival_to_utc` in `uscg.hpp`).
inline Json slot_to_utc(double unix_time, int slot_number,
                        std::optional<int> utc_hour = std::nullopt,
                        std::optional<int> utc_minute = std::nullopt) {
    if (slot_number < 0 || slot_number >= SLOTS_PER_FRAME) return Json();
    double slot_sec = slot_number * FRAME_DURATION_S / SLOTS_PER_FRAME;
    long long whole = static_cast<long long>(unix_time);

    if (utc_hour && utc_minute && *utc_hour >= 0 && *utc_hour <= 23 &&
        *utc_minute >= 0 && *utc_minute <= 59) {
        long long day_start = whole - (((whole % 86400) + 86400) % 86400);
        double tod = *utc_hour * 3600.0 + *utc_minute * 60.0 + slot_sec;
        double best = 0.0;
        double best_distance = -1.0;
        for (long long k : {-1LL, 0LL, 1LL}) {
            double candidate = static_cast<double>(day_start + k * 86400) + tod;
            double distance = std::abs(candidate - unix_time);
            if (best_distance < 0 || distance < best_distance) {
                best = candidate;
                best_distance = distance;
            }
        }
        if (best_distance > 43200.0) return Json();
        return Json::fixed(best, 3);
    }

    long long minute_start = whole - (((whole % 60) + 60) % 60);
    double best = 0.0;
    double best_distance = -1.0;
    for (int k : {-60, 0, 60}) {
        double candidate = static_cast<double>(minute_start + k) + slot_sec;
        double distance = std::abs(candidate - unix_time);
        if (best_distance < 0 || distance < best_distance) {
            best = candidate;
            best_distance = distance;
        }
    }
    if (best_distance > 30.0) return Json();
    return Json::fixed(best, 3);
}

}  // namespace aistut
