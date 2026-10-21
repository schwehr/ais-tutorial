"""asm.py — application-specific message (ASM) payloads, keyed by (DAC, FI).

Messages 6 and 8 (and 25/26, chapter 16) carry a 16-bit *application
identifier* — a 10-bit designated area code (DAC) and a 6-bit function
identifier (FI) — followed by data whose layout is defined by whoever owns
that DAC [ITU-R M.1371-5 Annex 5 §2.1 Table 23]. This module is the
registry that turns a known (dac, fi) pair into decoded fields. The message
decoders (msg_6.py, msg_8.py) call `decode(dac, fi, bits, offset)`; when it
returns a dictionary they attach it as `"application"`.

What is implemented, and from which source (chapter 9):

  (1, 0)    IFM 0 text telegram, 6-bit ASCII            M.1371-5 Annex 5 §5.1 Tables 25/26
  (1, 11)   Met/hydro, IMO SN/Circ.236 (2004)           Circ.236 trial message 1, as
                                                        described in AIVDM "Type 8(1/11)"
  (1, 31)   Met/hydro, IMO SN.1/Circ.289 (2010)         Circ.289 Annex Table 1.1
  (366, 34) USCG/NOAA right-whale zone notice (2008)    NOT published; reverse-engineered
                                                        from the encoder logs in
                                                        sample-data/RightwhaleAisProject

Everything else — DAC 0 test messages, the USCG encrypted DAC 366 FI 56/57,
the Seaway/PAWSS family, inland DAC 200 — stays as raw `data` bits in the
message object. `NAMES` lists a few more pairs than we decode so that the
chapter can print a catalogue; only DECODERS determines the output.

Conventions for the decoded fields
----------------------------------
* The ASM tables define a "not available" sentinel for almost every sensor
  value. Instead of a boolean companion per field (which would double the
  output) a not-available value is printed as `null`; the raw sentinel is
  not echoed. Enumerations keep the usual `_text` companion.
* Offsets in the comments are relative to the first bit *after* the
  application identifier (`o` in the code), so the same function serves
  message 6 (data starts at bit 88) and message 8 (bit 56).
* A decoder that finds too few bits returns `{"application_name": …,
  "error": "needs N data bits, have M"}` rather than raising, because a
  short ASM payload is still a perfectly good message 6 or 8.

Python notes for readers new to the language
--------------------------------------------
* `Callable[[Bits, int, int], dict]` (from `collections.abc`) is the type
  of "a function taking a Bits, an int and an int and returning a dict" —
  functions are values and can live in a dictionary, which is how the
  registry works. `type Decoder = ...` (PEP 695) gives that type a name.
* `(dac, fi)` tuples are hashable and therefore usable as dictionary keys.
"""

from collections.abc import Callable

from .bits import Bits
from .jsonout import Fixed

# --- small helpers ---------------------------------------------------------


def _na(raw: int, sentinel: int) -> int | None:
    """`raw` unless it equals the not-available sentinel, in which case None."""
    return None if raw == sentinel else raw


def _scaled(raw: int, sentinel: int, divisor: float, decimals: int,
            offset: float = 0.0) -> Fixed | None:
    """raw / divisor + offset with `decimals` places, or None if raw == sentinel."""
    if raw == sentinel:
        return None
    return Fixed(raw / divisor + offset, decimals)


TENDENCY: dict[int, str] = {0: "steady", 1: "decreasing", 2: "increasing", 3: "not available"}

# WMO 306 code table 4.201 as reproduced in Circ.289 Table 1.1.
PRECIPITATION: dict[int, str] = {
    0: "reserved", 1: "rain", 2: "thunderstorm", 3: "freezing rain",
    4: "mixed/ice", 5: "snow", 6: "reserved", 7: "not available",
}

# Beaufort scale, Circ.289 Table 1.2 (the same table is referenced by Circ.236).
SEA_STATE: dict[int, str] = {
    0: "calm (flat)", 1: "light air (ripples)", 2: "light breeze (small wavelets)",
    3: "gentle breeze (large wavelets)", 4: "moderate breeze (small waves)",
    5: "fresh breeze (moderate waves)", 6: "strong breeze (large waves)",
    7: "near gale (sea heaps up)", 8: "gale (moderately high waves)",
    9: "strong gale (high waves)", 10: "storm (very high waves)",
    11: "violent storm (exceptionally high waves)", 12: "hurricane (huge waves)",
    13: "not available", 14: "reserved", 15: "reserved",
}

ICE: dict[int, str] = {0: "no", 1: "yes", 2: "reserved", 3: "not available"}


# --- (1, 0) IFM 0: text telegram ------------------------------------------
#
# M.1371-5 Annex 5 §5.1, Table 25 (in message 6) and Table 26 (in message 8):
#
#      o+0      1  acknowledge required flag (0 for broadcasts)
#      o+1-11  11  text sequence number; 0 = sequence numbers not used
#      o+12..   n  text, 6-bit ASCII (Table 47), n a multiple of 6
#      last 0/2/4/6 spare bits for byte alignment (NOTE 1: a 6-bit spare
#               looks like a '@' character)

def decode_text_telegram(bits: Bits, o: int, data_bits: int) -> dict:
    text_bits = (data_bits - 12) // 6 * 6
    return {
        "ack_required": bits.flag(o),
        "text_sequence_number": bits.uint(o + 1, 11),
        "text": bits.text(o + 12, text_bits),
        "text_chars": text_bits // 6,
        "spare_bits": data_bits - 12 - text_bits,
    }


# --- (1, 11) IMO SN/Circ.236 meteorological and hydrological data ---------
#
# 296 data bits (352 in a message 8). Circ.236 "trial message 1"; the free
# description is AIVDM "Type 8(1/11)". Note latitude BEFORE longitude, and
# the offset ("add -60") rather than two's-complement encoding of the
# temperatures and water level. The not-available value of every field is
# its all-ones maximum ("the highest available binary value").
#
#      o+0-23    24  latitude, 1/1000 min, signed; 0x7FFFFF = n/a
#      o+24-48   25  longitude, 1/1000 min, signed; 0xFFFFFF = n/a
#      o+49-53    5  UTC day 1-31; 31 = n/a
#      o+54-58    5  UTC hour 0-23; 31 = n/a
#      o+59-64    6  UTC minute 0-59; 63 = n/a
#      o+65-71    7  average wind speed, knots; 127 = n/a
#      o+72-78    7  wind gust, knots; 127 = n/a
#      o+79-87    9  wind direction, degrees true; 511 = n/a
#      o+88-96    9  wind gust direction; 511 = n/a
#      o+97-107  11  air temperature, 0.1 °C, value/10 - 60; 2047 = n/a
#      o+108-114  7  relative humidity, %; 127 = n/a
#      o+115-124 10  dew point, 0.1 °C, value/10 - 20; 1023 = n/a
#      o+125-133  9  air pressure, hPa, value + 800; 511 = n/a
#      o+134-135  2  pressure tendency
#      o+136-143  8  horizontal visibility, 0.1 NM; 255 = n/a
#      o+144-152  9  water level, 0.1 m, value/10 - 10; 511 = n/a
#      o+153-154  2  water level trend
#      o+155-162  8  surface current speed, 0.1 kn; 255 = n/a
#      o+163-171  9  surface current direction; 511 = n/a
#      o+172-179  8  current speed 2, 0.1 kn; 255 = n/a
#      o+180-188  9  current direction 2; 511 = n/a
#      o+189-193  5  current measuring depth 2, m; 31 = n/a
#      o+194-201  8  current speed 3
#      o+202-210  9  current direction 3
#      o+211-215  5  current measuring depth 3
#      o+216-223  8  significant wave height, 0.1 m; 255 = n/a
#      o+224-229  6  wave period, s; 63 = n/a
#      o+230-238  9  wave direction; 511 = n/a
#      o+239-246  8  swell height, 0.1 m; 255 = n/a
#      o+247-252  6  swell period, s; 63 = n/a
#      o+253-261  9  swell direction; 511 = n/a
#      o+262-265  4  sea state, Beaufort; 13 = n/a
#      o+266-275 10  water temperature, 0.1 °C, value/10 - 10; 1023 = n/a
#      o+276-278  3  precipitation type; 7 = n/a
#      o+279-287  9  salinity, 0.1 ‰; 511 = n/a
#      o+288-289  2  ice
#      o+290-295  6  spare

MET_HYDRO_236_BITS = 296


def decode_met_hydro_236(bits: Bits, o: int, data_bits: int) -> dict:
    lat_raw = bits.sint(o, 24)
    lon_raw = bits.sint(o + 24, 25)
    tendency = bits.uint(o + 134, 2)
    level_trend = bits.uint(o + 153, 2)
    sea_state = bits.uint(o + 262, 4)
    precip = bits.uint(o + 276, 3)
    ice = bits.uint(o + 288, 2)
    return {
        "lat": None if lat_raw == 0x7FFFFF else Fixed(lat_raw / 60000.0, 6),
        "lon": None if lon_raw == 0xFFFFFF else Fixed(lon_raw / 60000.0, 6),
        "day": _na(bits.uint(o + 49, 5), 31),
        "hour": _na(bits.uint(o + 54, 5), 31),
        "minute": _na(bits.uint(o + 59, 6), 63),
        "wind_speed_knots": _na(bits.uint(o + 65, 7), 127),
        "wind_gust_knots": _na(bits.uint(o + 72, 7), 127),
        "wind_dir_deg": _na(bits.uint(o + 79, 9), 511),
        "wind_gust_dir_deg": _na(bits.uint(o + 88, 9), 511),
        "air_temp_c": _scaled(bits.uint(o + 97, 11), 2047, 10.0, 1, -60.0),
        "humidity_pct": _na(bits.uint(o + 108, 7), 127),
        "dew_point_c": _scaled(bits.uint(o + 115, 10), 1023, 10.0, 1, -20.0),
        "pressure_hpa": None if bits.uint(o + 125, 9) == 511 else bits.uint(o + 125, 9) + 800,
        "pressure_tendency": tendency,
        "pressure_tendency_text": TENDENCY[tendency],
        "visibility_nm": _scaled(bits.uint(o + 136, 8), 255, 10.0, 1),
        "water_level_m": _scaled(bits.uint(o + 144, 9), 511, 10.0, 1, -10.0),
        "water_level_trend": level_trend,
        "water_level_trend_text": TENDENCY[level_trend],
        "current_speed_knots": _scaled(bits.uint(o + 155, 8), 255, 10.0, 1),
        "current_dir_deg": _na(bits.uint(o + 163, 9), 511),
        "current_2_speed_knots": _scaled(bits.uint(o + 172, 8), 255, 10.0, 1),
        "current_2_dir_deg": _na(bits.uint(o + 180, 9), 511),
        "current_2_depth_m": _na(bits.uint(o + 189, 5), 31),
        "current_3_speed_knots": _scaled(bits.uint(o + 194, 8), 255, 10.0, 1),
        "current_3_dir_deg": _na(bits.uint(o + 202, 9), 511),
        "current_3_depth_m": _na(bits.uint(o + 211, 5), 31),
        "wave_height_m": _scaled(bits.uint(o + 216, 8), 255, 10.0, 1),
        "wave_period_s": _na(bits.uint(o + 224, 6), 63),
        "wave_dir_deg": _na(bits.uint(o + 230, 9), 511),
        "swell_height_m": _scaled(bits.uint(o + 239, 8), 255, 10.0, 1),
        "swell_period_s": _na(bits.uint(o + 247, 6), 63),
        "swell_dir_deg": _na(bits.uint(o + 253, 9), 511),
        "sea_state": sea_state,
        "sea_state_text": SEA_STATE[sea_state],
        "water_temp_c": _scaled(bits.uint(o + 266, 10), 1023, 10.0, 1, -10.0),
        "precipitation": precip,
        "precipitation_text": PRECIPITATION[precip],
        "salinity_ppt": _scaled(bits.uint(o + 279, 9), 511, 10.0, 1),
        "ice": ice,
        "ice_text": ICE[ice],
        "spare": bits.uint(o + 290, 6),
    }


# --- (1, 31) IMO SN.1/Circ.289 meteorological and hydrographic data -------
#
# 304 data bits (360 in a message 8). Circ.289 Annex Table 1.1. Longitude
# now comes first, temperatures are two's complement, water level has
# centimetre resolution, and "not available" is a distinct value per field.
#
#      o+0-24    25  longitude, 1/1000 min, signed; 181° (10 860 000) = n/a
#      o+25-48   24  latitude, 1/1000 min, signed; 91° (5 460 000) = n/a
#      o+49       1  position accuracy (1 = high, < 10 m)
#      o+50-54    5  UTC day 1-31; 0 = n/a
#      o+55-59    5  UTC hour 0-23; 24 = n/a
#      o+60-65    6  UTC minute 0-59; 60 = n/a
#      o+66-72    7  average wind speed, knots; 126 = ≥ 126; 127 = n/a
#      o+73-79    7  wind gust, knots; 126 = ≥ 126; 127 = n/a
#      o+80-88    9  wind direction, degrees true; 360 = n/a
#      o+89-97    9  wind gust direction; 360 = n/a
#      o+98-108  11  air temperature, 0.1 °C, two's complement; -1024 = n/a
#      o+109-115  7  relative humidity, %; 101 = n/a
#      o+116-125 10  dew point, 0.1 °C, two's complement; 501 = n/a
#      o+126-134  9  air pressure: 0 = ≤ 799 hPa, 1-401 = 800-1200, 402 = ≥ 1201; 511 = n/a
#      o+135-136  2  pressure tendency
#      o+137-144  8  visibility: MSB = "greater than" flag, low 7 bits in 0.1 NM; 127 = n/a
#      o+145-156 12  water level, 0.01 m, value/100 - 10; 4001 = n/a
#      o+157-158  2  water level trend
#      o+159-166  8  surface current speed, 0.1 kn; 251 = ≥ 25.1; 255 = n/a
#      o+167-175  9  surface current direction; 360 = n/a
#      o+176-183  8  current speed 2
#      o+184-192  9  current direction 2
#      o+193-197  5  current measuring level 2, m; 31 = n/a
#      o+198-205  8  current speed 3
#      o+206-214  9  current direction 3
#      o+215-219  5  current measuring level 3
#      o+220-227  8  significant wave height, 0.1 m; 251 = ≥ 25.1; 255 = n/a
#      o+228-233  6  wave period, s; 63 = n/a
#      o+234-242  9  wave direction; 360 = n/a
#      o+243-250  8  swell height
#      o+251-256  6  swell period
#      o+257-265  9  swell direction
#      o+266-269  4  sea state, Beaufort; 13 = n/a
#      o+270-279 10  water temperature, 0.1 °C, two's complement; 501 = n/a
#      o+280-282  3  precipitation type; 7 = n/a
#      o+283-291  9  salinity, 0.1 ‰; 501 = ≥ 50.1; 510 = n/a; 511 = sensor not available
#      o+292-293  2  ice
#      o+294-303 10  spare

MET_HYDRO_289_BITS = 304
LON_1000_NOT_AVAILABLE = 181 * 60000   # 10 860 000
LAT_1000_NOT_AVAILABLE = 91 * 60000    #  5 460 000


def _pressure_289(raw: int) -> int | None:
    """Circ.289 air pressure: 1-401 map to 800-1200 hPa; the ends are clamped."""
    if raw == 511:
        return None
    if raw == 0:
        return 799          # "pressure 799 hPa or less"
    if raw >= 402:
        return 1201         # "pressure 1201 hPa or greater" (403-510 reserved)
    return raw + 799


def decode_met_hydro_289(bits: Bits, o: int, data_bits: int) -> dict:
    lon_raw = bits.sint(o, 25)
    lat_raw = bits.sint(o + 25, 24)
    tendency = bits.uint(o + 135, 2)
    vis_raw = bits.uint(o + 137, 8)
    level_trend = bits.uint(o + 157, 2)
    sea_state = bits.uint(o + 266, 4)
    precip = bits.uint(o + 280, 3)
    salinity_raw = bits.uint(o + 283, 9)
    ice = bits.uint(o + 292, 2)
    return {
        "lon": None if lon_raw == LON_1000_NOT_AVAILABLE else Fixed(lon_raw / 60000.0, 6),
        "lat": None if lat_raw == LAT_1000_NOT_AVAILABLE else Fixed(lat_raw / 60000.0, 6),
        "position_accuracy": bits.uint(o + 49, 1),
        "day": _na(bits.uint(o + 50, 5), 0),
        "hour": _na(bits.uint(o + 55, 5), 24),
        "minute": _na(bits.uint(o + 60, 6), 60),
        "wind_speed_knots": _na(bits.uint(o + 66, 7), 127),
        "wind_gust_knots": _na(bits.uint(o + 73, 7), 127),
        "wind_dir_deg": _na(bits.uint(o + 80, 9), 360),
        "wind_gust_dir_deg": _na(bits.uint(o + 89, 9), 360),
        "air_temp_c": _scaled(bits.sint(o + 98, 11), -1024, 10.0, 1),
        "humidity_pct": _na(bits.uint(o + 109, 7), 101),
        "dew_point_c": _scaled(bits.sint(o + 116, 10), 501, 10.0, 1),
        "pressure_hpa": _pressure_289(bits.uint(o + 126, 9)),
        "pressure_tendency": tendency,
        "pressure_tendency_text": TENDENCY[tendency],
        "visibility_nm": _scaled(vis_raw & 0x7F, 127, 10.0, 1),
        "visibility_greater_than": (vis_raw >> 7) == 1,
        "water_level_m": _scaled(bits.uint(o + 145, 12), 4001, 100.0, 2, -10.0),
        "water_level_trend": level_trend,
        "water_level_trend_text": TENDENCY[level_trend],
        "current_speed_knots": _scaled(bits.uint(o + 159, 8), 255, 10.0, 1),
        "current_dir_deg": _na(bits.uint(o + 167, 9), 360),
        "current_2_speed_knots": _scaled(bits.uint(o + 176, 8), 255, 10.0, 1),
        "current_2_dir_deg": _na(bits.uint(o + 184, 9), 360),
        "current_2_depth_m": _na(bits.uint(o + 193, 5), 31),
        "current_3_speed_knots": _scaled(bits.uint(o + 198, 8), 255, 10.0, 1),
        "current_3_dir_deg": _na(bits.uint(o + 206, 9), 360),
        "current_3_depth_m": _na(bits.uint(o + 215, 5), 31),
        "wave_height_m": _scaled(bits.uint(o + 220, 8), 255, 10.0, 1),
        "wave_period_s": _na(bits.uint(o + 228, 6), 63),
        "wave_dir_deg": _na(bits.uint(o + 234, 9), 360),
        "swell_height_m": _scaled(bits.uint(o + 243, 8), 255, 10.0, 1),
        "swell_period_s": _na(bits.uint(o + 251, 6), 63),
        "swell_dir_deg": _na(bits.uint(o + 257, 9), 360),
        "sea_state": sea_state,
        "sea_state_text": SEA_STATE[sea_state],
        "water_temp_c": _scaled(bits.sint(o + 270, 10), 501, 10.0, 1),
        "precipitation": precip,
        "precipitation_text": PRECIPITATION[precip],
        "salinity_ppt": None if salinity_raw >= 510 else Fixed(salinity_raw / 10.0, 1),
        "salinity_sensor_available": salinity_raw != 511,
        "ice": ice,
        "ice_text": ICE[ice],
        "spare": bits.uint(o + 294, 10),
    }


# --- (366, 34) USCG/NOAA right-whale zone notice, 2008 form ---------------
#
# NOT in any published specification. The layout below was reverse-
# engineered (chapter 9) from the encoder's own debug output in
# sample-data/RightwhaleAisProject/20081015-boston-pog/20081015-pog-update.log
# ("set_from_bits zonemsg w/ dac fi / dac 0 10 366 / fi 10 16 34 /
# 16 21 1 / 21 28 0 / 28 33 15 / 33 38 15 / 38 44 10 / 44 52 10 /
# zonebits (76)") and the BMS request lines that carry the payload bits
# next to their parameters, then checked against the messages received on
# the air. The data is a sequence of 112-bit records; the Cornell queue
# manager packed up to five records into one message, the CCOM script one.
#
#      r+0-4     5  zone id (1-10 = acoustic buoy TSS-AB01..AB10)
#      r+5-11    7  zone type: 0 = right whales not observed, 1 = present
#      r+12-16   5  start UTC day
#      r+17-21   5  start UTC hour
#      r+22-27   6  start UTC minute
#      r+28-35   8  duration in units of 10 minutes (1 = 10 min, 144 = 24 h)
#      r+36     76  one circular zone:
#          +0     1  shape selector (always 0 in the data; see chapter 9)
#          +1-28 28  longitude, 1/10 000 min, signed (as message 1)
#          +29-55 27  latitude,  1/10 000 min, signed
#          +56-71 16  radius, metres (always 9 260 = 5 NM in the data)
#          +72-75  4  spare

WHALE_RECORD_BITS = 112
WHALE_ZONE_TYPE: dict[int, str] = {0: "right whales not observed", 1: "right whales present"}


def decode_whale_notice(bits: Bits, o: int, data_bits: int) -> dict:
    notices: list[dict] = []
    r = o
    while bits.has(r, WHALE_RECORD_BITS):
        zone_type = bits.uint(r + 5, 7)
        duration_raw = bits.uint(r + 28, 8)
        z = r + 36
        notices.append({
            "zone_id": bits.uint(r, 5),
            "zone_type": zone_type,
            "zone_type_text": WHALE_ZONE_TYPE.get(zone_type, "unknown"),
            "start_day": bits.uint(r + 12, 5),
            "start_hour": bits.uint(r + 17, 5),
            "start_minute": bits.uint(r + 22, 6),
            "duration_raw": duration_raw,
            "duration_minutes": duration_raw * 10,
            "shape": bits.uint(z, 1),
            "lon": Fixed(bits.sint(z + 1, 28) / 600000.0, 6),
            "lat": Fixed(bits.sint(z + 29, 27) / 600000.0, 6),
            "radius_m": bits.uint(z + 56, 16),
            "spare": bits.uint(z + 72, 4),
        })
        r += WHALE_RECORD_BITS
    return {"notices": notices, "trailing_bits": o + data_bits - r}


# --- the registry ----------------------------------------------------------

Decoder = Callable[[Bits, int, int], dict]   # a type alias (plain assignment works on Python 3.10+)

# (dac, fi) -> (name, minimum data bits, decoder)
DECODERS: dict[tuple[int, int], tuple[str, int, Decoder]] = {
    (1, 0): ("IFM 0 text telegram", 12, decode_text_telegram),
    (1, 11): ("IMO Circ.236 meteorological and hydrological data", MET_HYDRO_236_BITS, decode_met_hydro_236),
    (1, 31): ("IMO Circ.289 meteorological and hydrographic data", MET_HYDRO_289_BITS, decode_met_hydro_289),
    (366, 34): ("USCG/NOAA right whale zone notice (2008)", WHALE_RECORD_BITS, decode_whale_notice),
}

# Names only — for the catalogue in chapter 9; these are NOT decoded.
NAMES: dict[tuple[int, int], str] = {
    (1, 22): "IMO Circ.289 area notice (broadcast)",
    (1, 23): "IMO Circ.289 area notice (addressed)",
    (366, 22): "USCG area notice (broadcast)",
    (366, 56): "USCG encrypted AIS position (EAIS)",
    (366, 57): "USCG encrypted AIS static data (EAIS)",
}


def decode(dac: int, fi: int, bits: Bits, offset: int) -> dict | None:
    """Decode the application data that starts at `offset`.

    Returns None when (dac, fi) is unknown. For a known pair the result
    always carries `application_name`; if the payload is shorter than the
    layout needs it carries an `error` text instead of the fields.
    """
    entry = DECODERS.get((dac, fi))
    if entry is None:
        return None
    name, need, func = entry
    data_bits = len(bits) - offset
    out: dict = {"application_name": name}
    if data_bits < need:
        out["error"] = f"needs {need} data bits, have {data_bits}"
        return out
    out.update(func(bits, offset, data_bits))
    return out
