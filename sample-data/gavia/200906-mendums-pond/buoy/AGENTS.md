# AGENTS.md — `sample-data/gavia/200906-mendums-pond/buoy/`

Raw buoy logs, 1 June 2009 18:00–21:16 UTC. Every line is

```
<SOURCE> \t DATA \t <ISO-8601 host time>      <sentence>\r\r\n
```

| File | Lines | Size | Sentences |
|---|---|---|---|
| `RTK1_GPS_2009_6_1.log` | 78 832 | 9.0 MB | 1 Hz cycle of `$GPGGA` (lat/lon with 8 decimals of minutes: `4310.17550652,N,07103.68294260,W`, fix quality 1 or 4 = RTK fixed, altitude `71.356,M`), `$PTNL,GGK,…,EHT38.439,M` (Trimble proprietary time/position/quality with ellipsoid height), `$GPGST` (pseudorange error statistics: RMS, semi-major/minor, orientation, σlat, σlon, σalt), `$GPGSV` (2 per cycle), `$GPZDA` (date/time), `$GPGSA`. One stray compass line leaked in. |
| `RTK2_GPS_2009_6_1.log` | 29 593 | 3.3 MB | Same receiver type configured for a smaller set: `$PTNL,GGK`, `$GPGST`, `$GPZDA` only. A few compass lines leaked in. |
| `RTK1_ATT_2009_6_1.log` | 112 739 | 8.8 MB | Compass at ~10 Hz: `$C141.7P-10.6R-0.5T16.9*1A` = **C**ompass heading 141.7°, **P**itch −10.6°, **R**oll −0.5°, **T**emperature 16.9 °C, standard NMEA `*hh` XOR checksum. Not a standard NMEA sentence (no talker/formatter, no commas) but checksummed the same way. |
| `RTK2_ATT_2009_6_1.log` | 113 442 | 8.6 MB | Same for buoy 2. |
| `Makefile` | 46 B | `./nmea.py RTK1_GPS_2009_6_1.log.ggk` — a reference to a processing script and derived `.ggk` file that are not in the tree. |

Teaching uses: the NMEA checksum applies to *anything* between `$` and `*`;
`$GPGST` as an example of a sentence beginning readers rarely meet; RTK fix
quality values in `GGA` field 6 (`4` = RTK fixed, `5` = RTK float).
