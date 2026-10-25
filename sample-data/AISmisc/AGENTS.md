# AGENTS.md — `sample-data/AISmisc/`

Miscellaneous AIS captures from 2007–2009, mostly from the CCOM / Stellwagen
Bank right-whale AIS project and from Class B transceiver experiments. Each
subdirectory has its own `AGENTS.md`.

## Files directly in this directory

| File | Size | Contents |
|---|---|---|
| `Sealand2007-06-27-001414.nmea` | 97 KB, `\r\r\n` | Serial capture (no logger banner, no timestamps) from an **SRT-based Class B transceiver** at sea south of Georges Bank (40.6°N 67.7°W, 9.3 kn on 27 Jun 2007). Mix of `$PSRT` proprietary status lines (`$PSRT,010,,,338022762` = own MMSI; `LED`; `ADC` eight analogue readings), `!AIVDO` **message 18** own reports (`B52GAJP0…`), 85 `!AIVDM` from nearby ships, `$GPRMC`, `$GPGBS` (RAIM), and `$AIALR` alarms with the `AIS: Tx malfunction / Antenna VSWR fault / Rx channel N malfunction` text set. Good for `!AIVDO` message 18 and for `$AIALR`. |
| `classb.uscg.19.nmea` | 251 B | Two-line **message 19** (extended Class B) extracted from the N-AIS feed on 3 Dec 2007, in USCG format with `s`, `r` and epoch fields. A two-fragment sentence pair (`!AIVDM,2,1,6,A,…` / `!AIVDM,2,2,6,A,…`) — the tutorial's canonical multi-sentence example. |
| `maeastptlght.ais.2008-03-12` | 330 KB | SR162G receiver at **Eastern Point Light, Gloucester MA** (station `maeastptlght`), one of the whale-project shore receivers. 1 680 `$GPRMC`/`!AIVDO` pairs (the receiver's own GPS as a pseudo message 1), 425 `!AIVDM`, 3 `$AITXT`. |
| `mahghlndlght.log.2008-03-17` | 165 KB | **Highland Light, Truro MA** (station `mahghlndlght`): a *Yacht AIS* receiver (banner `RX YACHT AIS Receiver v1.3`) plus an L-3 base station on the same line, hence `!BSVDO` **message 4** own reports, `$BSALR`, `$BSADS`. |
| `sbnms_tss.zip` | 16 KB | ESRI shapefiles (`old_TSS_*`, `New_TSS_*` in GCS and UTM) of the Boston Traffic Separation Scheme before and after its July 2007 re-routing to reduce right-whale strikes. Context for the whale notices in `../RightwhaleAisProject/`. |

## Subdirectories

| Directory | Date | Summary |
|---|---|---|
| `2008-03-24-sbnms-whale-test/` | 24 Mar 2008 | First at-sea test of whale notice reception: R/V *Auk* SR162G log and a Nauticast transceiver log (`rm808`). |
| `2008-03-27-bs-send-test/` | 27 Mar 2008 | Transmitting from an L-3 **base station** in the CCOM office and listening with an SR162: `!BSVDO`, `$BSALR`, `$BSADS`, L-3 `$PL3COMFLASH`. |
| `2008-03-28-bs-send-test/` | 28 Mar 2008 | Empty (files not preserved). |
| `200905-Cocheco-classb-config/` | 1 May 2009 | Configuring the Class B unit for UNH's R/V *Cocheco*: `$AISSD`/`$AIVSD` static-data set sentences, `$PSRT`, `!AIVDO` message 18 before and after an MMSI was programmed. |
