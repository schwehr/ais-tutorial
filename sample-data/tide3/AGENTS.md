# AGENTS.md — `sample-data/tide3/`

The **UNH Coastal Marine Laboratory (CML)** site in New Castle NH, at the
mouth of the Piscataqua River — station id `nhcml`. `README.txt` says the
logger was installed in the upstairs office on 28 Sep 2007 by Kurt Schwehr,
running

```
serial_logger.py -b 9600
serial_logger.py --port=/dev/ttyS2 --log-prefix=ais- --uscg-format --station-id nhcml --timeout=300 --mark-timeouts
```

and notes that on 18 Feb 2008 "Andy M. switched which tube/sensor is being
used for tide" and Ubuntu 7.10 was installed. Only the AIS half survived:

```
tide3/
├── README.txt
├── ais/     870 files ais-2008-02-26 … ais-2011-05-26 (139 MB) incl. 59 .bz2 duplicates
└── tide/    empty
```

CML looks straight at the Portsmouth Harbor approaches, so this is the best
**clean, long-duration coastal receiver** in the collection: three-plus years
of message 1/3/5, base stations (4), Class B (18, 24 — common from 2009 on,
e.g. `B5MiOp…`/`H5MiOp…` in May 2011), a few 19, 21, and large volumes of
**message 8 DAC 366 FI 56/57** — USCG *encrypted* AIS test traffic in late
2010 (`85PH4C1Kf2:S<g5FbF6URDOBFO8@…`, MMSI 3692…, undecodable by design).
Also contains message 12/13 (addressed safety text + ack) in Jan–Mar 2010 and
message 10 (UTC inquiry) in March 2010.

Use `ais/AGENTS.md` for the file format and a per-type finder's guide.
