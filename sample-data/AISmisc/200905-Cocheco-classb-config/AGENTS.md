# AGENTS.md — `sample-data/AISmisc/200905-Cocheco-classb-config/`

1 May 2009, Kurt's office at CCOM: programming an **SRT-based Class B "CS"
transceiver** for UNH's small research boat R/V *Cocheco* and watching the
result with a second receiver. Both files are `serial_logger.py` captures.

| File | Size | Station / port | Contents |
|---|---|---|---|
| `transceiver2009-05-01` | 281 KB | `kurt-office2`, `/dev/ttyS2` — the Class B unit itself | 2 975 `$PSRT` proprietary lines (`SRM`, `010`, `LED`, `ADC`, `VER`, `SNO` serial number, `SWF`…), 1 173 `!AIVDO` **message 18** own reports — first with MMSI 0 (`B0000003wk?8mP=18D3Q3wv5kP06`) and later with the programmed MMSI — 171 `$AIALR`, 102 `$AITXT,01,01,60,AIS: Internal GNSS not in use`, 6 `$GPTXT` (`ANTSTATUS=OK`), and the configuration sentences `$AISSD,NONE@@@,R/V COCHECO@@@@@@@@@,007,004,03,01,1,AI` (call sign, name, dimensions A/B/C/D, DTE, source) and `$AIVSD,056,00.0,0000,@@@…` (ship type 56 … voyage data). |
| `log.2009-05-01` | 225 KB | `kurt-office`, `/dev/ttyS1` — SR162G receiver | 1 287 `$GPRMC`/`!AIVDO` pairs only; the receiver's own position pseudo-report. Shows that the SR162G's `!AIVDO` is **not** an AIS transmission, just the receiver's GPS re-encoded as message 1 (`13tfD@?P00Js@gVHcVO7P?wN0000`). |

Teaching uses: message 18 and the `$AISSD`/`$AIVSD` sentences used to load
static data into a transponder (the data that later appears in **message 24**
parts A and B); `@` padding in NMEA text fields; MMSI `000000000` before
configuration.
