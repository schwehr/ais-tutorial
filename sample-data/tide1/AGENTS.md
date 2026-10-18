# AGENTS.md — `sample-data/tide1/`

The **Jackson Estuarine Laboratory (JEL)** site on Great Bay, Durham/Newmarket
NH — station id `nhjel`. A Linux box logged an AIS receiver on `/dev/ttyS2`
(38 400 baud) and a tide gauge on `/dev/ttyS0` (9 600 baud) with
`serial_logger.py`. The site was set up for the UNH "AIS as a data channel for
environmental sensors" work (tide data via AIS binary messages; see
`UNH-AisBinMsgRfc-20061020.pdf` in the open-manual specs), so the two streams
are meant to be joined by time.

```
tide1/
├── ais/           816 files ais-2007-02-17 … ais-2009-05-10 (34 MB) + README.txt + ttyS22007-02-18
├── tide/          353 files log.2007-02-17 … log.2008-02-04 (306 MB!)
└── OldTideData/   8 files 2006_05_24.TXT … 2007_01_23.TXT (17 MB)
```

See the `AGENTS.md` in each subdirectory for formats.

**Big caveat on the AIS data.** The `nhjel` receiver (or its serial link) was
unreliable: a very large share of its `!AIVDM` lines are *truncated* (payloads
of 6–15 characters instead of 28) or *bit-flipped*, yet carry a checksum that
matches the garbled text. Example from `ais/ais-2008-11-05`: 555 lines have a
28-character payload (valid message 1/3), 184 have 56 (two-fragment 5), but
179 have only 15 and others 6, 8 or 11. If you tally "message types" by the
first payload character you will see impossible types 28–63. The tutorial
uses this directory deliberately to teach **payload-length validation**.

Great Bay is far up the Piscataqua estuary, so the traffic is thin: a handful
of ships at the Portsmouth terminals (MMSI `15N…`, `15NLiJ…`), the occasional
tug, plus base-station and Class B traffic from 2009 on.
