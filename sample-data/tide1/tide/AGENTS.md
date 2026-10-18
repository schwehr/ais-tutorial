# AGENTS.md — `sample-data/tide1/tide/`

353 daily tide-gauge logs `log.YYYY-MM-DD`, 17 Feb 2007 – 4 Feb 2008, 306 MB
in total (about 1.4 MB per day — the gauge talks every 2 min but the logger
also wrote a `# <epoch>` line before every output line in 2007, and in the
largest files the gauge was echoing continuously). Captured from `/dev/ttyS0`
at 9 600 baud with `serial_logger.py` (`TIMEOUT: 15.0`).

The gauge prints a plain-text block, not NMEA:

```
Date/Time:  7.12.25  0:57:29
01 Reference             736
02 Water level             0.28 m
03 Water temperature       0.02 Deg.C
```

* `Date/Time` is `Y.MM.DD HH:MM:SS` with a **one-digit year** (7 = 2007).
* `Reference` is an instrument counter/reference value.
* `Water level` in metres (relative to the sensor), `Water temperature` in °C.

In 2007 each of these lines is preceded by a `# 1171735687.67` logger
timestamp line; in 2008 the timestamps are absent and only the banner is
there. Water level sampled every two minutes is the series that was packed
into the UNH experimental AIS binary "water level" message (see the
open-manual spec `UNH-AisBinMsgRfc-20061020.pdf`).
