"""nmea.py — the NMEA 0183 sentence layer (chapter 1).

An NMEA 0183 sentence (IEC 61162-1 §7) looks like

    $GPGGA,000429.000,3601.5522,N,10947.9359,W,1,7,4.21,12491.0,M,-20.5,M,,*6D
    ^^     ^^^                                                              ^^
    talker formatter                 comma-separated data fields        checksum

* `$` (or `!` for encapsulated data such as AIS) starts the sentence.
* The next two characters are the *talker* (GP = GPS, AI = AIS mobile
  station, BS = base station, AN = AtoN, …), then a 3-letter *formatter*
  (GGA, RMC, VDM, …). Proprietary sentences start with `P` + a 3-letter
  manufacturer code (`PMTK`, `PSRT`, `PFEC`) and have no fixed formatter.
* Fields are separated by commas; an empty field is allowed and means
  "no data".
* `*` followed by two hex digits is the checksum: the XOR of every byte
  between `$`/`!` and `*` (exclusive). IEC 61162-1 §7.2.3.
* The sentence ends with CR LF and may be at most 82 characters including
  `$` and CR LF (§7.1.1). Real logs often break this; we do not enforce it.

The USCG / serial_logger convention of appending extra comma fields *after*
the checksum (`…*6D,rplane_to_sd,1197072048.14`) is also handled here:
those fields are returned separately as `after_checksum`.

Python notes
------------
* `@dataclass` auto-generates `__init__`, `__repr__` and `__eq__` for a
  class that is just a bundle of named fields — ideal for parsed records.
* `Optional[int]` (or `int | None`) means "an int, or None if absent".
* `str.partition('*')` splits a string at the first '*' into three parts.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional

from .jsonout import Fixed


def checksum(body: str) -> int:
    """XOR of all characters in `body` (the text between '$' and '*').

    >>> hex(checksum("GPGSV,1,1,00"))
    '0x79'
    """
    value = 0
    for ch in body:
        value ^= ord(ch)
    return value


@dataclass
class Sentence:
    """One parsed NMEA sentence, before any formatter-specific decoding."""

    start: str                       # '$' or '!'
    talker: str                      # 'GP', 'AI', 'P' for proprietary ...
    formatter: str                   # 'GGA', 'VDM', 'MTK010' ...
    fields: list[str]                # data fields *after* the address field
    checksum_given: Optional[int]    # value written in the sentence, or None
    checksum_ok: bool                # True only if present and matching
    after_checksum: list[str] = field(default_factory=list)   # USCG extras
    raw: str = ""

    @property
    def address(self) -> str:
        """The address field, e.g. 'GPGGA' or 'AIVDM'."""
        return self.talker + self.formatter


class NmeaError(ValueError):
    """Raised for text that is not an NMEA sentence at all."""


def parse_sentence(line: str) -> Sentence:
    """Split one line into its NMEA parts and verify the checksum.

    Leading/trailing whitespace and CR/LF are ignored. Characters before the
    `$`/`!` (such as a `file-name:` prefix from grep) are discarded; TAG
    blocks (chapter 4) must be stripped by the caller first.
    """
    line = line.strip()
    start_at = min((i for i in (line.find("$"), line.find("!")) if i >= 0), default=-1)
    if start_at < 0:
        raise NmeaError("no '$' or '!' in line")
    line = line[start_at:]
    start = line[0]

    body, star, tail = line[1:].partition("*")
    given: Optional[int] = None
    ok = False
    extras: list[str] = []
    if star:
        hexpart, _, rest = tail.partition(",")
        try:
            given = int(hexpart[:2], 16)
        except ValueError:
            given = None
        ok = given is not None and given == checksum(body)
        if rest:
            extras = rest.split(",")
    # If there was no '*' we still parse the fields but checksum_ok is False.

    address, *fields = body.split(",")
    if len(address) < 3:
        raise NmeaError(f"address field too short: {address!r}")
    if address.startswith("P"):          # proprietary: P + 3-char maker code
        talker, formatter = "P", address[1:]
    else:
        talker, formatter = address[:2], address[2:]
    return Sentence(start, talker, formatter, fields, given, ok, extras, line)


# --- Helpers for the GPS formatters ----------------------------------------


def parse_latitude(ddmm: str, hemisphere: str) -> Optional[float]:
    """'4312.2805','N' -> 43.204675 ; empty -> None.

    NMEA writes latitude as DDMM.MMMM: two digits of degrees followed by
    minutes with a decimal fraction. Longitude uses three degree digits
    (DDDMM.MMMM); see parse_longitude.
    """
    if not ddmm:
        return None
    degrees = int(ddmm[:2])
    minutes = float(ddmm[2:])
    value = degrees + minutes / 60.0
    return -value if hemisphere == "S" else value


def parse_longitude(dddmm: str, hemisphere: str) -> Optional[float]:
    """'07052.8474','W' -> -70.88079"""
    if not dddmm:
        return None
    degrees = int(dddmm[:3])
    minutes = float(dddmm[3:])
    value = degrees + minutes / 60.0
    return -value if hemisphere == "W" else value


def parse_utc_time(hhmmss: str) -> Optional[str]:
    """'000429.000' -> '00:04:29.000' (kept as text; no date is available)."""
    if len(hhmmss) < 6:
        return None
    return f"{hhmmss[0:2]}:{hhmmss[2:4]}:{hhmmss[4:]}"


def parse_date(ddmmyy: str) -> Optional[str]:
    """'081207' -> '2007-12-08'. Two-digit years: NMEA predates Y2K, we
    assume 2000–2099 which is right for every file in this tutorial."""
    if len(ddmmyy) != 6:
        return None
    return f"20{ddmmyy[4:6]}-{ddmmyy[2:4]}-{ddmmyy[0:2]}"


def opt_float(s: str) -> Optional[float]:
    return float(s) if s else None


def fixed(value: Optional[float], decimals: int) -> Optional[Fixed]:
    """Wrap a float for printing with `decimals` places (see jsonout.py)."""
    return None if value is None else Fixed(value, decimals)


def opt_int(s: str) -> Optional[int]:
    return int(s) if s else None


# --- The four GPS sentences of chapter 1 -----------------------------------
#
# Field numbers below are 1-based positions *after* the address field, which
# is how the NMEA standard numbers them.

GGA_FIX_QUALITY = {
    0: "invalid", 1: "GPS fix (SPS)", 2: "DGPS fix", 3: "PPS fix",
    4: "RTK fixed", 5: "RTK float", 6: "estimated (dead reckoning)",
    7: "manual input", 8: "simulation",
}


def parse_gga(s: Sentence) -> dict:
    """GGA — Global positioning system fix data (IEC 61162-1 §8.3.x / NMEA 0183 v3 §GGA).

    1 UTC time hhmmss.ss        6 fix quality (table above)
    2 latitude ddmm.mmmm        7 number of satellites in use
    3 N/S                        8 HDOP
    4 longitude dddmm.mmmm      9 antenna altitude, 10 'M'
    5 E/W                       11 geoidal separation, 12 'M'
                                13 age of DGPS data (s), 14 DGPS station id
    """
    f = s.fields + [""] * (14 - len(s.fields))     # pad short sentences
    quality = opt_int(f[5])
    return {
        "sentence": "GGA",
        "time_utc": parse_utc_time(f[0]),
        "lat": fixed(parse_latitude(f[1], f[2]), 6),
        "lon": fixed(parse_longitude(f[3], f[4]), 6),
        "fix_quality": quality,
        "fix_quality_text": GGA_FIX_QUALITY.get(quality, "unknown") if quality is not None else None,
        "satellites_used": opt_int(f[6]),
        "hdop": fixed(opt_float(f[7]), 2),
        "altitude_m": fixed(opt_float(f[8]), 1),
        "geoid_separation_m": fixed(opt_float(f[10]), 1),
        "dgps_age_s": fixed(opt_float(f[12]), 1),
        "dgps_station": f[13] or None,
    }


GSA_MODE2 = {1: "no fix", 2: "2D", 3: "3D"}


def parse_gsa(s: Sentence) -> dict:
    """GSA — GNSS DOP and active satellites.

    1 mode: M = manual, A = automatic 2D/3D switching
    2 mode: 1 = fix not available, 2 = 2D, 3 = 3D
    3–14 PRN numbers of the (up to 12) satellites used in the solution
    15 PDOP  16 HDOP  17 VDOP
    """
    f = s.fields + [""] * (17 - len(s.fields))
    fix_mode = opt_int(f[1])
    return {
        "sentence": "GSA",
        "selection_mode": f[0] or None,
        "fix_mode": fix_mode,
        "fix_mode_text": GSA_MODE2.get(fix_mode) if fix_mode is not None else None,
        "satellites_used": [int(p) for p in f[2:14] if p],
        "pdop": fixed(opt_float(f[14]), 2),
        "hdop": fixed(opt_float(f[15]), 2),
        "vdop": fixed(opt_float(f[16]), 2),
    }


def parse_gsv(s: Sentence) -> dict:
    """GSV — GNSS satellites in view. One sentence carries up to 4 satellites.

    1 total number of GSV sentences in this group
    2 this sentence's number (1-based)
    3 total satellites in view
    then repeating groups of 4: PRN, elevation (deg), azimuth (deg), SNR (dB-Hz)
    Elevation/azimuth/SNR are empty when not tracked.
    """
    f = s.fields
    sats = []
    for i in range(3, len(f) - 3, 4):
        prn, elev, az, snr = f[i:i + 4]
        if prn:
            sats.append({"prn": int(prn), "elevation_deg": opt_int(elev),
                         "azimuth_deg": opt_int(az), "snr_db": opt_int(snr)})
    return {
        "sentence": "GSV",
        "total_sentences": opt_int(f[0]) if len(f) > 0 else None,
        "sentence_number": opt_int(f[1]) if len(f) > 1 else None,
        "satellites_in_view": opt_int(f[2]) if len(f) > 2 else None,
        "satellites": sats,
    }


RMC_MODE = {"A": "autonomous", "D": "differential", "E": "estimated",
            "M": "manual", "S": "simulator", "N": "not valid"}


def parse_rmc(s: Sentence) -> dict:
    """RMC — Recommended minimum specific GNSS data.

    1 UTC time   2 status A = valid, V = warning (no fix)
    3,4 latitude   5,6 longitude   7 SOG knots   8 COG degrees true
    9 date ddmmyy   10,11 magnetic variation deg, E/W
    12 mode indicator (NMEA 2.3+)   13 navigational status (NMEA 4.1+)
    """
    f = s.fields + [""] * (13 - len(s.fields))
    var = opt_float(f[9])
    if var is not None and f[10] == "W":
        var = -var
    return {
        "sentence": "RMC",
        "time_utc": parse_utc_time(f[0]),
        "status": f[1] or None,
        "valid": f[1] == "A",
        "lat": fixed(parse_latitude(f[2], f[3]), 6),
        "lon": fixed(parse_longitude(f[4], f[5]), 6),
        "sog_knots": fixed(opt_float(f[6]), 2),
        "cog_deg": fixed(opt_float(f[7]), 2),
        "date": parse_date(f[8]),
        "magnetic_variation_deg": fixed(var, 1),
        "mode": f[11] or None,
        "mode_text": RMC_MODE.get(f[11]) if f[11] else None,
    }


GPS_PARSERS = {"GGA": parse_gga, "GSA": parse_gsa, "GSV": parse_gsv, "RMC": parse_rmc}
