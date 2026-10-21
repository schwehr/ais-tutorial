"""decode.py — command-line driver: NMEA/AIS log lines in, JSON lines out.

    python3 decode.py FILE [FILE...]        # or '-' for standard input

For every input line we print at most one JSON object (see jsonout.py for
why the JSON is hand-rolled). The same program exists in C++, Rust and
TypeScript, and the test-suite checks that all four print identical text.

Output shapes
-------------
GPS sentence          {"line":7,"sentence":"GGA", ...fields...}
AIS message           {"line":9,"ais":{...decoded...},"channel":"A","talker":"AI",
                       "own_ship":false,"payload_bits":168,"fragments":1,
                       "uscg":{...},"tagblock":{...}}
                      ("uscg"/"tagblock" only when present)
unknown formatter     {"line":3,"address":"PMTK010","skipped":"no parser"}
problem               {"line":5,"error":"checksum mismatch","raw":"..."}
Logger comments (# ...) and blank lines print nothing.
"""

import sys
from collections.abc import Callable

from .bits import Bits
from .common import DecodeError
from .jsonout import dumps
from .nmea import GPS_PARSERS, NmeaError, parse_sentence
from .tagblock import split_tag_block
from .uscg import parse_uscg_fields
from .vdm import Assembler, Message, parse_vdm

# One module per message type (or family). Each exposes decode(bits) -> dict.
from . import (msg_1_2_3, msg_4_11, msg_5, msg_6, msg_7_13, msg_8, msg_9,
               msg_10, msg_12, msg_14, msg_15, msg_16, msg_17, msg_18,
               msg_19, msg_20, msg_21, msg_22, msg_23, msg_24, msg_25,
               msg_26, msg_27)

DECODERS: dict[int, Callable[[Bits], dict]] = {
    1: msg_1_2_3.decode, 2: msg_1_2_3.decode, 3: msg_1_2_3.decode,
    4: msg_4_11.decode, 11: msg_4_11.decode,
    5: msg_5.decode,
    6: msg_6.decode,
    7: msg_7_13.decode, 13: msg_7_13.decode,
    8: msg_8.decode,
    9: msg_9.decode,
    10: msg_10.decode,
    12: msg_12.decode,
    14: msg_14.decode,
    15: msg_15.decode,
    16: msg_16.decode,
    17: msg_17.decode,
    18: msg_18.decode,
    19: msg_19.decode,
    20: msg_20.decode,
    21: msg_21.decode,
    22: msg_22.decode,
    23: msg_23.decode,
    24: msg_24.decode,
    25: msg_25.decode,
    26: msg_26.decode,
    27: msg_27.decode,
}


def decode_ais(msg: Message) -> dict:
    """Turn an assembled VDM payload into a dictionary (or an error entry)."""
    try:
        bits = Bits.from_payload(msg.payload, msg.fill_bits)
    except ValueError as e:
        return {"error": str(e)}
    if len(bits) < 6:
        return {"error": f"payload too short: {len(bits)} bits", "payload_bits": len(bits)}
    msg_type = bits.uint(0, 6)
    decoder = DECODERS.get(msg_type)
    out: dict = {"payload_bits": len(bits)}
    if decoder is None:
        out["error"] = f"unknown message type {msg_type}"
        out["msg_type"] = msg_type
        return out
    try:
        out["ais"] = decoder(bits)
    except DecodeError as e:
        out["error"] = str(e)
        out["msg_type"] = msg_type
    except NotImplementedError:
        out["ais"] = {"msg_type": msg_type, "not_implemented": True}
    return out


def process_line(line: str, lineno: int, asm: Assembler) -> dict | None:
    """Handle one text line. Returns a dict to print, or None to stay quiet."""
    stripped = line.strip()
    if not stripped or stripped.startswith("#"):
        return None
    out: dict = {"line": lineno}

    try:
        tag, rest = split_tag_block(stripped)
    except ValueError as e:
        out["error"] = str(e)
        out["raw"] = stripped
        return out
    if tag is not None:
        out["tagblock"] = tag.to_dict()

    try:
        s = parse_sentence(rest)
    except NmeaError as e:
        out["error"] = str(e)
        out["raw"] = stripped
        return out
    if s.after_checksum:
        out["uscg"] = parse_uscg_fields(s.after_checksum)
    if not s.checksum_ok:
        out["error"] = "checksum mismatch" if s.checksum_given is not None else "no checksum"
        out["raw"] = s.raw
        return out

    if s.formatter in ("VDM", "VDO"):
        try:
            frag = parse_vdm(s)
        except ValueError as e:
            out["error"] = str(e)
            out["raw"] = s.raw
            return out
        msg = asm.add(frag)
        if msg is None:
            return None                    # waiting for more fragments
        out.update(decode_ais(msg))
        out["channel"] = msg.channel
        out["talker"] = msg.talker
        out["own_ship"] = msg.own_ship
        out["fragments"] = len(msg.fragments)
        return out

    parser = GPS_PARSERS.get(s.formatter) if s.talker != "P" else None
    if parser is None:
        out["address"] = s.address
        out["skipped"] = "no parser"
        return out
    # A GPS sentence with a valid checksum can still carry a non-numeric
    # field (a receiver firmware bug, or a logger that re-computes
    # checksums). int()/float() raise ValueError; report it like any other
    # bad line instead of letting it abort the whole run.
    try:
        out.update(parser(s))
    except ValueError as e:
        out["error"] = str(e)
        out["raw"] = s.raw
    return out


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__)
        return 2
    asm = Assembler()
    lineno = 0
    for name in argv:
        stream = sys.stdin if name == "-" else open(name, encoding="utf-8", errors="replace")
        with stream:
            for line in stream:
                lineno += 1
                result = process_line(line, lineno, asm)
                if result is not None:
                    try:
                        print(dumps(result))
                    except BrokenPipeError:      # e.g. piped into `head`
                        return 0
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
