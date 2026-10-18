# AGENTS.md — `sample-data/2007hydro/`

AIS logs recorded around the **US Hydro 2007** conference (Norfolk,
Virginia, 14–18 May 2007). Two kinds of equipment were on the bench:

* **Smart Radio SR162** dual-channel AIS receivers (`!AIVDM` plus a periodic
  `$AITXT,01,01,91,FREQ,2087,2088` status line announcing the two tuned
  channels — 2087/2088 are the ITU channel numbers for AIS 1 and AIS 2).
* An **L-3 Communications ProTec** Class A transponder whose maintenance
  serial ports spit out internal debug text (`Radio: Prog Rx 1, AB Digit = 2…`,
  `TdmaProc:`, `DscMain:`, `VdlExec:` …) in between normal IEC 61162 sentences.

The area is the Hampton Roads / Elizabeth River complex, one of the busiest
AIS environments on the US East Coast (Navy, container terminals, pilots), so
these files are rich in message types that quiet receivers never see:
**15 (interrogation), 16 (assignment), 20 (DLM), 22 (channel management)** and
base-station **4** reports, plus many binary **8** messages with DAC 55/366.

| File | Size | Format | Contents |
|---|---|---|---|
| `norfolk-log.ais` | 21 KB (302 lines) | `sentence,<epoch>` | Short SR162 sample on 14 May. First line is a rare `$AIACA` (channel assignment) sentence. Good small file for message 1/3/4/5/20. |
| `kurts_log_BGsRoomOvernightsheraton_norfolk.ais` | 3.2 MB (52 015 lines) | `sentence,<epoch>`, `\r\r\n` | Overnight SR162 capture from a hotel room, 16–17 May. ~2 165 message 5 and ~2 445 message 20. |
| `va-2007-05-15` | 2.2 MB | `sentence,<epoch>` | SR162 in Portsmouth VA, 15 May, header-less. |
| `va-portsmouth-sr162-2007-05-15` | 3.5 MB | logger | Same receiver with logger banners; ~54 900 `!AIVDM`, 520 `$AITXT`. Contains **205 message 15** interrogations and **6 677 message 8 DAC 55 FI 58** binaries (undocumented, probably encrypted). |
| `va-portsmouth-sr162-2007-05-17` | 3.0 MB | logger | 17 May; ~8 800 message 4 base-station reports and 2 839 message 20. |
| `va-portsmouth-sr162-2007-05-18` | 3.3 MB | logger | 18 May. |
| `sr162-noaa-norfolk-telcom2007-05-14` | 771 KB | logger | SR162 on the NOAA side of the conference, 14 May; 10 947 `!AIVDM`, 106 `$AITXT`. Includes **message 8 DAC 117 FI 16** examples. |
| `protec-util12007-05-14` | 4.5 MB | logger, **binary junk present** | ProTec "utility" port 1: 18 921 `!AIVDM`, 1 276 `!AIVDO` (own ship), 156 `$AIALR` alarms (`AIS: Tx malfunction`, `Antenna VSWR exceeds limit`, `Rx channel 1 malfunction`…), 121 `!AIBRF`, 3 `$AITXT,…,007,AIS: UTC clock lost`, 241 **message 22** channel management, 8 **message 2**. The transponder had no antenna or GPS, hence the alarms. |
| `protec-util22007-05-14` | 3 KB | logger | A few seconds of ProTec boot output on the second utility port. |
| `protec-pi2007-05-14` | 790 KB | logger, binary | ProTec "PI" (presentation interface) port: mostly debug lines (`Proc_RMC: Lat = …`, `NTS_MAP`). Few usable NMEA sentences. |

Notes:

* The `<epoch>` suffix on these early files is the only metadata; there is no
  `r<station>` field. Parsers must accept both 1 and 2 trailing fields.
* `!AIBRF` is a non-standard L-3 sentence (the payload looks like an MMSI and
  a small counter). Do not try to decode it; mention it as "vendor noise".
* `grep` reports the ProTec files as binary; use `grep -a` or Python with
  `errors="replace"`.
