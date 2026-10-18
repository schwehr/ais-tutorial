# AGENTS.md — `sample-data/AISmisc/2008-03-24-sbnms-whale-test/`

24 March 2008: the first at-sea test of the right-whale notice broadcasts in
Stellwagen Bank National Marine Sanctuary (SBNMS). Two logs, both
`serial_logger.py` captures with `r<station>,<epoch>` suffixes.

| File | Size | Station | Contents |
|---|---|---|---|
| `log-auk-sr162g.2008-03-24` | 4.9 MB | `auk` (NOAA R/V *Auk*) | SR162G receiver on the boat: 28 270 `$GPRMC`/`!AIVDO` pairs (GPS track from the Portsmouth NH dock, `4312.2764,N,07052.8432,W`, down to Massachusetts Bay, `4211.8,N,07040.2,W`, 19 kn), 5 585 `!AIVDM` (2 673 message 1, 2 631 message 3, **42 message 8** — the whale notices, e.g. `85Mwp?QKf<SwEVd2iRvk@Ti6mS0G0>BBg3<aDON<Gce?Qom3nPc6`), 53 `$AITXT`. |
| `log-rm808.2008-03-24` | 3.7 MB, binary junk | `masbnms-rm808` (SBNMS shore site) | An ACR **Nauticast** Class A transponder (`$PNAUTSI` proprietary) used as a receiver: 36 066 `!AIVDO` (own ship `352HH<wP?w<tSF0l4Q@>4?wp0P00` — note the "no position" 181°/91° pattern because it has no GPS), 6 801 `$AIALR` (ids 005, 025, 032, 035: `RX CHANNEL 70 MALFUNCTION`, `EXTERNAL EPFS LOST`, `HEADING LOST INVALID`, `NO VALID ROT INFORMATION`), 5 444 `!AIVDM`. |

Teaching uses: `!AIVDO` from a transponder with no sensors (default/unavailable
field values), the `$AIALR` alarm sentence, and a real-world track in `$GPRMC`
to overlay with AIS positions.
