# AGENTS.md — `sample-data/2010-11-l3-aton/`

Bench tests of an **L-3 Communications AIS Aid-to-Navigation (AtoN) station**
("L3 AT91 AtoN Project, Rev 1.20B", with an SRT Marine AtoN radio board
inside) in the CCOM office at UNH, on 4 September 2010 and 2–5 November 2010.
Four serial ports were logged at once with `serial_logger.py`; the port number
is in the file name (`l3-2`, `l3-3`, `l3-4`) and the station id
(`rccom-office-l3-N`). An independent **SR162G receiver** (`ccom-sr162g-*`,
station `rccom-office`) listened to what the AtoN actually transmitted.

This is the place to get **message 21 (AtoN report)** samples, including
*virtual* and *off-position* flags, and the AtoN's own `!ANVDO` echo of what
it sent, framed with the NMEA 4.0 `AN` talker id. It also has **message 8
DAC 366** test broadcasts and the only `$ANADS` (AIS device status) and
`$ANZDA` sentences in the collection.

| File | Size | Port role | Contents |
|---|---|---|---|
| `ccom-sr162g-2010-09.aivdm` | 50 KB | SR162G receiver, Sept 2010 | `grep`-style output: each line is prefixed with `ccom-sr162g-2010-09-15:` or `…-16:` (the source file name) — strip everything up to the first `:` before parsing. 429 **message 21** reports from MMSI 003669739 (`E03Ovrh…`) and a handful of message 8 (`803Ovri…`). **Best small file for message 21.** |
| `ccom-sr162g-2010-11-02` | 1.1 MB | SR162G receiver, 2 Nov | 6 122 `$GPRMC`/`!AIVDO` pairs (the receiver's own GPS, stuck on a stale 2009 fix at 22°N 114°E — a factory default) and 62 `!AIVDM`: 29 message 21, 18 message 8 (`803OvrhFqH2P3V…`, DAC 366), a few message 1/5 from real ships in Portsmouth NH. |
| `ccom-sr162g-2010-11-03`, `-04`, `-05` | < 300 B | | Only logger banners (`# STOP LOGGING`, `# Log roll over`). Useful to show that empty logs exist. |
| `ccom-l3-2-2010-09-04`, `ccom-l3-2-2010-11-02` | 10 MB, 5.2 MB | AtoN debug console, 115 200 baud | Firmware boot banner and verbose internal trace (`UbxMsg: AID-HUI Rcvd`, `M13Svc: *** Cfg Lat/Lon [42 4.4428 N, 70 12.3202 W]`, `PrcGGA:`, `MkTxRs:` …). Contains binary bytes. No NMEA. |
| `ccom-l3-3-2010-09-04`, `ccom-l3-3-2010-11-02` | 254 KB, 187 KB | AtoN NMEA output #1 | Mixed: `$ANALR,…,007,V,V,AIS: UTC Lost`, `$ANZDA`, `!ANVDO` message 21 (`E00VT:QVIgPb7W…`, MMSI 993672…), interleaved with radio-board status lines `%0030 0001`, `%0031 0011`, `Slot Number: 500`, `Successful Modulation`. |
| `ccom-l3-4-2010-09-04` | 797 KB | SRT radio board bootloader console | Repeated `* SRT MARINE ATON BOOTLOADER` banners and `%0026 0011` status; no NMEA. |
| `ccom-l3-4-2010-11-02` | 116 KB | AtoN NMEA output #2 | Clean: 913 `!ANVDO` message 21 (channel field `X`, `Y`, `A` or `B`!), 217 `$ANZDA`, 90 `$ANADS,L3 AIS ID,,A,4,I,N`, 84 `$ANALR`. **Best file for `!ANVDO` and non-standard channel codes.** |

Notes:

* The channel field of `!ANVDO` holds `X`/`Y` on some lines. This is outside
  the `A`/`B`/`1`/`2` the standards describe; a robust parser must not crash.
* MMSI `003669739` is a US coast-station style identity (`00` + MID 366); the
  AtoN under test also used `993672…` (`99` + MID 367 = AtoN format).
* Compare `!ANVDO` lines (what the unit *sent*) with the SR162G `!AIVDM` lines
  (what was *received*) by matching timestamps — a nice end-to-end exercise.
