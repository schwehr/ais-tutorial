"""uscg.py — the US Coast Guard "N-AIS" receive metadata (chapter 5).

The USCG Research & Development Center's AIS network (and Kurt Schwehr's
`serial_logger.py`, which copied the idea) appends extra comma-separated
fields *after* the NMEA checksum:

    !AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02,s34486,d-074,T25.49701619,x78124,r003669946,1223901985
                                                   └ s: signal strength, raw receiver units (0–65535)
                                                          └ d: signal strength in dBm
                                                                 └ T: time of arrival, seconds into the UTC minute
                                                                              └ x: receiver's sentence counter
                                                                                      └ r: receiving station id
                                                                                                   └ UNIX time (s)

The format was never published as a standard; it is described in the GPSD
AIVDM.adoc "U.S. Coast Guard Extended AIVDM" section and in the AISUser
manual (output format 2, "proprietary USCG R&D Center format"). Field
order varies and most fields are optional; only the station id and the
final time are always present. Known prefixes:

    s   received signal strength indicator (RSSI), receiver-specific units
    d   signal strength, dBm (negative number)
    T   time of arrival within the minute (fractional seconds)
    S   slot number (0–2249) in which the message was received
    x   sentence counter from the receiver
    r   receiving station id (`b` is also seen for base stations)
    (none) a bare number: UNIX time in seconds, possibly with a fraction
"""


from .jsonout import Fixed


def _is_ascii_digits(s: str) -> bool:
    """Non-empty and only '0'..'9'. Plain `str.isdigit()` also accepts
    '²' and Arabic-Indic digits, which the other three ports do not."""
    return s.isascii() and s.isdigit()


def parse_uscg_fields(extras: list[str]) -> dict:
    """Interpret the fields that followed the checksum.

    Unknown or malformed items are kept verbatim in `unparsed` so that no
    information is silently lost.
    """
    out: dict = {}
    unparsed: list[str] = []
    for item in extras:
        if not item:
            continue
        code, rest = item[0], item[1:]
        try:
            if code == "s" and _is_ascii_digits(rest.lstrip("-")):
                out["rssi"] = int(rest)
            elif code == "d" and _is_ascii_digits(rest.lstrip("-")):
                out["signal_dbm"] = int(rest)
            elif code == "T":
                out["time_of_arrival_s"] = Fixed(float(rest), 8)
            elif code == "S" and _is_ascii_digits(rest):
                out["slot"] = int(rest)
            elif code == "x" and _is_ascii_digits(rest):
                out["sentence_counter"] = int(rest)
            elif code in ("r", "b") and rest:
                out["station"] = rest
                out["station_kind"] = "base" if code == "b" else "receiver"
            elif _is_ascii_digits(item.replace(".", "", 1)):
                # A bare number is the UNIX receive time.
                out["time"] = Fixed(float(item), 2) if "." in item else int(item)
            else:
                unparsed.append(item)
        except ValueError:
            unparsed.append(item)
    if unparsed:
        out["unparsed"] = unparsed
    return out


def time_of_arrival_to_utc(unix_time: int, toa_s: float | None) -> Fixed | None:
    """Combine the coarse UNIX time with the fine `T` field.

    `T` is seconds past the start of the UTC minute measured by the
    receiver's clock, so the precise arrival time is "start of some UTC
    minute + T". The logging computer's `unix_time` tells us *which*
    minute: we try the minute containing `unix_time` and its neighbours
    and keep the candidate closest to `unix_time` (the log time is a few
    seconds late, and may have rolled into the next minute).
    """
    if toa_s is None:
        return None
    minute_start = unix_time - (unix_time % 60)
    candidates = [minute_start + k + toa_s for k in (-60, 0, 60)]
    precise = min(candidates, key=lambda t: abs(t - unix_time))
    if abs(precise - unix_time) > 30:
        return None
    return Fixed(precise, 3)
