#!/usr/bin/env python3
"""uscg_to_tagblock.py — rewrite USCG-format lines as NMEA 4.10 TAG block lines.

    python3 tools/uscg_to_tagblock.py < uscg.nmea > tagblock.nmea

This is what the USCG AISUser program does when its OutputFormat setting is
changed from 2 (USCG R&D Center format) to 1 (TAG block). It is a good
exercise because it forces you to decide what each metadata field *means*:

    r003669946  -> s:003669946   (source station)
    1223901985  -> c:1223901985  (UNIX time, seconds)
    s34486, d-074, T25.497, x78124 have no standard TAG block code; NMEA
    4.10 allows no private codes, so they are carried in the free text
    field t: as "s34486 d-074 T25.497 x78124" (at most 15 characters are
    allowed, so this is lossy — we keep the dBm and the time of arrival).

Multi-sentence groups get a g:<n>-<total>-<group id> parameter and an
n:<line> counter so that a downstream program can re-associate fragments
even if other sentences are interleaved.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))   # find aistut/

from aistut.nmea import parse_sentence           # noqa: E402
from aistut.tagblock import format_tag_block     # noqa: E402
from aistut.uscg import parse_uscg_fields        # noqa: E402


def convert(line: str, counter: int, group_ids: dict) -> str:
    s = parse_sentence(line)
    meta = parse_uscg_fields(s.after_checksum)
    params: dict[str, str] = {}
    if "station" in meta:
        params["s"] = str(meta["station"])
    if "time" in meta:
        params["c"] = str(int(float(str(meta["time"]))))
    extras = []
    if "signal_dbm" in meta:
        extras.append(f"d{meta['signal_dbm']}")
    if "time_of_arrival_s" in meta:
        extras.append(f"T{float(str(meta['time_of_arrival_s'])):.2f}")
    if extras:
        params["t"] = " ".join(extras)[:15]
    if s.formatter in ("VDM", "VDO") and s.fields[0] != "1":
        total, number, seq = s.fields[0], s.fields[1], s.fields[2]
        key = (s.talker, s.fields[3], seq)
        if number == "1" or key not in group_ids:
            group_ids[key] = len(group_ids) + 1
        params["g"] = f"{number}-{total}-{group_ids[key]}"
    params["n"] = str(counter)
    # Keep only the sentence proper (drop the USCG suffix after the checksum).
    body = s.raw
    star = body.find("*")
    sentence = body[: star + 3] if star >= 0 else body
    return format_tag_block(params) + sentence


def main() -> None:
    group_ids: dict = {}
    counter = 0
    for raw in sys.stdin:
        raw = raw.strip()
        if not raw or raw.startswith("#"):
            continue
        counter += 1
        print(convert(raw, counter, group_ids))


if __name__ == "__main__":
    main()
