"""commstate.py — TDMA communication state schedule and timing helpers (chapter 17).

Spec: ITU-R M.1371-5 Annex 2 §3.1 (synchronisation and frame structure),
§3.3.4 (SOTDMA and ITDMA slot allocation), §3.3.7.2.2 Tables 18/19 (SOTDMA
communication state), and §3.3.7.3.2 Table 20 (ITDMA communication state).

Eight AIS message types end with a communication state:

    1, 2      168 bits   bits 149-167 (19)   always SOTDMA
    3         168 bits   bits 149-167 (19)   always ITDMA
    4, 11     168 bits   bits 149-167 (19)   always SOTDMA
    9         168 bits   bits 148-167 (20)   selector at 148 + 19-bit state
    18        168 bits   bits 148-167 (20)   selector at 148 + 19-bit state
    26     <=1064 bits   last 20 bits        selector + 19-bit state

`msg_1_2_3.py` decodes the 19-bit SOTDMA and ITDMA bit fields into a dict;
this module re-exports those decoders and adds the frame-arithmetic and
clock-recovery functions that turn raw slot numbers, offsets, and UTC
sub-messages into physical schedules and UTC arrival times.
"""

from .jsonout import Fixed
from .msg_1_2_3 import SYNC_STATE, decode_itdma, decode_sotdma

__all__ = [
    "SLOTS_PER_FRAME",
    "FRAME_DURATION_S",
    "SLOT_DURATION_S",
    "TMO_MIN",
    "TMO_MAX",
    "UTC_HOUR_NA",
    "UTC_MINUTE_NA",
    "SYNC_STATE",
    "decode_sotdma",
    "decode_itdma",
    "slot_to_seconds",
    "next_sotdma_slot",
    "next_itdma_slot",
    "slot_to_utc",
]

# Annex 2 §3.2.1 Table 10: one 60 s frame has 2 250 slots (26.667 ms each).
SLOTS_PER_FRAME = 2250
FRAME_DURATION_S = 60.0
SLOT_DURATION_S = FRAME_DURATION_S / SLOTS_PER_FRAME

# Annex 2 §3.3.4.4 Table 16: a newly allocated SOTDMA slot is assigned a
# random integer time-out between TMO_MIN (3) and TMO_MAX (7) frames.
TMO_MIN = 3
TMO_MAX = 7

# Annex 2 §3.3.7.2.2 Table 19: in the `slot_timeout == 1` UTC hour/minute
# sub-message, hour 0..23 and minute 0..59 are valid; 24 and 60 mean n/a.
UTC_HOUR_NA = 24
UTC_MINUTE_NA = 60


def slot_to_seconds(slot_number: int) -> Fixed | None:
    """Convert a slot number (`0..2249`) to seconds into the UTC minute.

    Each TDMA frame is 60 s divided into 2 250 slots, so slot `k` begins at
    `k * 60 / 2250 = k * 4 / 150` seconds past the minute [Annex 2 §3.2.1].
    Returns `None` if `slot_number` is outside `0..2249`.
    """
    if not (0 <= slot_number < SLOTS_PER_FRAME):
        return None
    return Fixed(slot_number * FRAME_DURATION_S / SLOTS_PER_FRAME, 3)


def next_sotdma_slot(slot_number: int, slot_offset: int) -> int | None:
    """Return the slot in the *next* frame announced by a SOTDMA `slot_offset`.

    When a SOTDMA stream reaches `slot_timeout == 0`, its 14-bit sub-message
    is a `slot_offset` relative to the current slot on the same channel
    [Annex 2 §3.3.7.2.2 Table 19]:
      * `slot_offset == 0` means the slot is de-allocated after this
        transmission (`None`);
      * any non-zero offset selects `(slot_number + slot_offset) % 2250` in
        the next frame (because offsets are modulo 2 250, values below 2 250
        can represent negative shifts).
    """
    if not (0 <= slot_number < SLOTS_PER_FRAME) or slot_offset <= 0:
        return None
    return (slot_number + slot_offset) % SLOTS_PER_FRAME


def next_itdma_slot(slot_number: int, slot_offset: int) -> int | None:
    """Return the slot of the next transmission announced by an ITDMA `slot_offset`.

    In ITDMA (`msg_1_2_3.decode_itdma`), `slot_offset` is already expanded to
    `slot_increment` (`0..8191`) or `slot_increment + 8192` when `slots_raw`
    is 5, 6, or 7 [Annex 2 §3.3.7.3.2 Table 20]. An offset of `0` means this
    is the last transmission for the slot (`None`); otherwise the next
    transmission on the same channel lands in `(slot_number + slot_offset) % 2250`
    (`(slot_number + slot_offset) // 2250` frames later).
    """
    if not (0 <= slot_number < SLOTS_PER_FRAME) or slot_offset <= 0:
        return None
    return (slot_number + slot_offset) % SLOTS_PER_FRAME


def slot_to_utc(
    unix_time: float,
    slot_number: int,
    utc_hour: int | None = None,
    utc_minute: int | None = None,
) -> Fixed | None:
    """Recover the UTC transmission timestamp from a logger clock and SOTDMA state.

    `slot_number` (`0..2249`, carried when `slot_timeout` is 2, 4, or 6) gives
    the sub-minute offset `slot_sec = slot_number * 60 / 2250` [Annex 2 Table 19].

    * When `utc_hour` (`0..23`) and `utc_minute` (`0..59`) are also known —
      from a nearby `slot_timeout == 1` sub-message — the full time-of-day
      within the 86 400 s UTC day is fixed. We test the UTC day containing
      `unix_time` and its two neighbours (`-86400`, `0`, `+86400`) and keep
      the candidate closest to `unix_time`. This resolves clock offsets of
      many minutes or hours (such as the 52.6 s fast logger in `ais-2008-06-12`
      where a naive nearest-minute match lands in the wrong minute).
    * When `utc_hour` or `utc_minute` is `None` (or outside `0..23` / `0..59`),
      we fall back to the nearest minute within 30 s of `unix_time` — the
      same rule as `uscg.time_of_arrival_to_utc`.
    """
    if not (0 <= slot_number < SLOTS_PER_FRAME):
        return None
    slot_sec = slot_number * FRAME_DURATION_S / SLOTS_PER_FRAME
    whole = int(unix_time)
    if (
        utc_hour is not None
        and utc_minute is not None
        and 0 <= utc_hour <= 23
        and 0 <= utc_minute <= 59
    ):
        day_start = whole - (whole % 86400)
        tod = utc_hour * 3600 + utc_minute * 60 + slot_sec
        candidates = [day_start + k * 86400 + tod for k in (-1, 0, 1)]
        best = min(candidates, key=lambda t: abs(t - unix_time))
        if abs(best - unix_time) > 43200:
            return None
        return Fixed(best, 3)

    minute_start = whole - (whole % 60)
    candidates = [minute_start + k + slot_sec for k in (-60, 0, 60)]
    best = min(candidates, key=lambda t: abs(t - unix_time))
    if abs(best - unix_time) > 30:
        return None
    return Fixed(best, 3)
