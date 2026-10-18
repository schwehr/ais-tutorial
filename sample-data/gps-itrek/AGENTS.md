# AGENTS.md — `sample-data/gps-itrek/`

Plain **NMEA 0183 GPS** logs from an i.Trek Bluetooth GPS receiver (MediaTek
MTK chipset; the Mac serial port is `/dev/tty.iBT-GPS-SPPslave-1`, 38 400 baud).
No AIS here. This is the gentlest introduction to NMEA sentence structure and
the first dataset used by the tutorial.

Every file is a `serial_logger.py` capture (see `../AGENTS.md` for the file
shape). The receiver emits a fixed 1 Hz cycle of

```
$GPGGA  fix data (time, lat, lon, fix quality, satellites, HDOP, altitude, geoid separation)
$GPGSA  active satellites and DOP values
$GPGSV  satellites in view (2–3 sentences per cycle)
$GPRMC  recommended minimum (time, status, lat, lon, SOG, COG, date, magnetic variation, mode)
```

plus, right after power-up, the proprietary sentences `$PMTK010,001` (MTK
"system startup" notification) and `$POLYN,…` (chipset
ephemeris/almanac/restart status lines **without checksums**).

| File | Size | Lines | STATIONID | Notes |
|---|---|---|---|---|
| `gps.log.2007-12-05.1` | 5.2 MB | 131 494 | `None` | Oldest logger format: a `# <epoch>` comment line *before* each sentence, no trailing fields. Durham NH area (43.2°N 70.88°W). Starts with `$PMTK010`/`$POLYN` cold-start lines and no-fix `GGA` (`fix quality 0`). |
| `i.Trek.gps.log.2007-12-05` | 10.9 MB | 114 519 | `kurt_office_159` | Static receiver in a CCOM office window, 3-D fixes (`GSA` mode 3). Good for "what does a stationary GPS look like". |
| `gps.log.2007-12-07` | 303 KB | 3 257 | `plane_to_sd` | Starts in Durham NH without a fix, then the beginning of an airline flight. |
| `gps.log.2007-12-08` | 148 KB | 1 700 | `plane_to_sd` | **Tutorial file.** Three logger restarts (three `# START LOGGING` banners). First segment: no fix with placeholder position `8960.0000,N,00000.0000,E` (an MTK quirk — latitude 89°60′!). From line ~646 the receiver locks on in flight at 12 491 m altitude over the US Southwest (36.0°N 109.8°W), SOG 347 kn, COG 245°. |
| `kurt-gps-log.2007-12-10` | 2.8 MB | 30 592 | `kurt_itrek` | Over the Pacific off California (34.3°N 123.1°W) heading south-east; ends on the ground. |
| `log.2007-12-17` | 608 KB | 7 527 | `kurt_itrek` | Mountain View CA (37.39°N 122.1°W). Includes a cold start showing `$POLYN,TIME,RESTART_OCCURRED`, `$POLYN,EPH`, `$POLYN,ALM`, `$POLYN,ION`, `$POLYN,UTC`. |

Things worth pointing out to readers:

* `GGA` field 6 (fix quality) is `0` for a long time at the start of most
  files, and `RMC` status is `V` (void) — a parser must cope with empty fields
  (`,,`) for HDOP, satellites, etc.
* `GSV` is a multi-sentence group (`$GPGSV,3,1,10,…`, `3,2,…`, `3,3,…`) and the
  last sentence is shorter; satellite elevation/azimuth are often empty.
* The appended `,r<station>,<epoch>` fields come *after* the `*hh` checksum, so
  the checksum must be computed only up to `*`.
* Altitude of `12491.0,M` on 2007-12-08 is a nice sanity check that your
  altitude parser is right.
