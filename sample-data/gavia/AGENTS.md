# AGENTS.md — `sample-data/gavia/`

Logs from CCOM's **Gavia** autonomous underwater vehicle programme. Only one
experiment survived: `200906-mendums-pond/`, a 1 June 2009 field day at Mendums
Pond (Barrington, NH, 43.17°N 71.06°W) where two instrumented surface buoys
carried Trimble RTK GPS receivers and an electronic compass. No AIS; this is
high-precision **GPS NMEA** (centimetre RTK, `$PTNL,GGK`, `$GPGST` error
statistics) and a minimal proprietary compass sentence, useful as a contrast
to the consumer-grade i.Trek logs in `../gps-itrek/`.

```
gavia/
├── 200906-mendums-pond/
│   ├── buoy/                 four logs + Makefile (see buoy/AGENTS.md)
│   └── mendums-pond-map-csv/ empty
└── navigator/                empty
```

The log line format differs from every other directory here: a tab-separated
prefix `RTK1_GPS \t DATA \t 2009-06-01T18:00:52.507770` (source name, record
type, ISO-8601 host timestamp) followed by spaces and the raw sentence. Lines
end in `\r\r\n`.
