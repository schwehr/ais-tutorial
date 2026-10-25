# AGENTS.md — `sample-data/tide3/ais/`

870 daily logs `ais-YYYY-MM-DD` (811 plain + 59 `.bz2`) from the `nhcml`
receiver at the UNH Coastal Marine Lab, 26 Feb 2008 – 26 May 2011, 139 MB.
Every `.bz2` has a same-named plain file next to it with identical content
(Dec 2009 – Mar 2010); ignore the `.bz2` copies.

Format is the standard late `serial_logger.py` shape:

```
# START LOGGING UTC seconds since the epoch: 1305936034.716424
# SPEED:       38400
# PORT:        /dev/ttyS1
# TIMEOUT:     300.0
# STATIONID:   nhcml
# DAEMON MODE: True
# MARK: 1305936034.65
!AIVDM,1,1,,B,H5MiOp1:uJ0MDhJ0<P4hhDpLE80,2*33,rnhcml,1305936051.69
```

`# MARK:` lines are written every 300 s of silence. Typical day: 1 000–5 000
sentences. Finder's guide (file → what is in it):

| Want | File(s) |
|---|---|
| Class B 18 + 24 (A and B parts) | `ais-2011-05-21` … `ais-2011-05-26` (706 message 18 and 368 message 24 on 24 May) |
| Message 19 | `ais-2010-07-28`, `ais-2010-09-30` |
| Message 21 AtoN | `ais-2010-10-*` (virtual AtoN tests) |
| Message 12 / 13 safety text + ack | `ais-2010-01-30` (9 + 11), `ais-2010-03-06` |
| Message 10 UTC inquiry | `ais-2010-03-08`, `ais-2010-03-09` |
| Message 7 binary ack | `ais-2010-01-15` |
| Message 8 DAC 366 FI 56/57 (USCG encrypted) | `ais-2010-12-01` and the rest of Dec 2010 |
| Message 8 DAC 366 FI 34 whale notice | sporadic 2008–2009 (received from Boston-area transceivers at long range) |
| Message 6 DAC 0 FI 0 (unprogrammed addressed binary) | `ais-2010-12-27` |
| Message 25 single-slot binary | `ais-2011-05-26` (verify length!) |
| Long-range message 27 | `ais-2009-06-23` (verify) |

The receiver is clean compared with `../../tide1/ais/` but not perfect; the
usual rule applies — check the payload bit count against the message type.
