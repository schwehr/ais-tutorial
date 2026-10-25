# AGENTS.md — `sample-data/RightwhaleAisProject/2008-10-01-auk-test-cape-cod/`

1–2 October 2008: the NOAA R/V *Auk* steamed from Scituate past the Cape Cod
transceiver to confirm that whale notices were received at sea.
`README.txt` (86 B): "Testing the AIS transceiver on Cape Cod at the visitor
center from the NOAA R/V Auk".

| File | Size | Contents |
|---|---|---|
| `rvauk-sr162g2008-10-01` | 3.8 MB | `serial_logger.py` capture (station `rvauk-sr162g`, daemon mode) of the boat's SR162G: 19 685 `$GPRMC`/`!AIVDO` pairs tracking the boat (starts 42°12.1′N 70°41.9′W at 18 kn, 12:11 UTC; back at the dock 17:41 UTC), 4 367 `!AIVDM`, 44 `$AITXT`. |
| `hyperterminal/2008_10_1.txt` | 393 KB | HyperTerminal capture (`\r\r\n`, no timestamps) from the *Auk*'s own **Furuno** Class A transceiver: 7 548 `!AIVDO` (own position `15Pg8V0…`, MMSI 366… — note the channel field alternates `A`/`B`/empty), 369 `!AIVDM`, 52 `$AIALR`, 395 `$PFEC,idfnc,R,4` (Furuno proprietary). |
| `hyperterminal/2008_10_2.txt` | 625 KB | Continuation on 2 October, same mix (13 179 lines). |

Teaching uses: `!AIVDO` from a real Class A at speed (non-zero SOG/COG/ROT,
heading), comparing a vessel's own `VDO` with how a shore receiver saw it as
`VDM`, and a proprietary `$P…` sentence to be skipped gracefully.
