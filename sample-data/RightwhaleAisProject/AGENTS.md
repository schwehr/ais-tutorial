# AGENTS.md — `sample-data/RightwhaleAisProject/`

Working files from the 2008–2009 **Right Whale Listening Network / AIS
whale-notice project**: Cornell's Bioacoustics Research Program ran ten
auto-detection buoys (`TSS-AB01`…`TSS-AB10`) along the Boston Traffic
Separation Scheme; every few minutes the detection status of each buoy was
encoded into an **AIS binary broadcast (message 8, DAC 366, FI 34 — the
pre-IMO-289 USCG "zone/area notice")** and transmitted from shore AIS
transponders so that ships entering Stellwagen Bank NMS could see where whales
had been heard. CCOM (Kurt Schwehr) built the encoder; the transponders were
**CNS Systems VDL 6000** Class A units (proprietary `$PCNSS`/`$PCNSR`
sentences) driven by Cornell's "AIS Radio Interface" (ARI) and "Queue
Manager" (QM) Windows programs.

Everything needed to teach *sending* binary messages is here: the `BMS,…`
text requests handed to the radio software, the `!VTBBM`/`!AIBBM` broadcast
binary message sentences sent to the transponder, the `$AIABK` acknowledgement
back, and the `!AIVDM` lines received by independent receivers.

## Files in this directory

| File | Size | Contents |
|---|---|---|
| `20081013.log` | 49 MB (498 505 lines) | A **telnet capture of the N-AIS regional feed** (`Trying 99.178.167.126…` header, then raw `!AIVDM` lines in USCG format `…*hh,s26911,d-106,T35.00363275,x120385,r003669945,1224006993`), 13–14 Oct 2008, two Boston-area receivers `r003669945` and `r003669946`. 353 372 message 1, 35 859 message 4, 29 554 message 3, ~12 000 message 5, 8 155 message 8 (5 685 of them DAC 366 FI 34 whale notices), 32 message 6, plus 9, 12, 14, 18, 21, 24 and a few long-range 27. **The richest single file in the collection; the primary source for the USCG-metadata chapter.** |
| `20091009-capecod-hyperterm.txt` | 95 KB | HyperTerminal capture (no timestamps, `\r\n`) from the Cape Cod transponder a year later: `!AIVDO` message 1 own reports (`103OvriP…`, MMSI 003669739), `!AIVDM`, `$PCNSR,BFP,…` / `$PCNSR,BFS,1` "Blue Force" mode responses, `$AIALR`. |
| `cns-standard-dos.txt`, `cns-silent-dos.txt`, `cns-secure-dos.txt` | 33 B each | One-line `$PCNSS,BFP,…` commands (DOS line endings) that switch a VDL 6000 between *standard* (`1,1,1,1,1,1,1,1,1`), *silent* (all `0`) and *secure* (`1,1,0,…`) transmit modes. |

## Subdirectories

| Directory | Date | Summary |
|---|---|---|
| `2008-10-01-auk-test-cape-cod/` | 1–2 Oct 2008 | R/V *Auk* sailing past the Cape Cod transponder to verify reception; SR162G log and HyperTerminal captures from a Furuno unit (`$PFEC`). |
| `20081003-AIS/` | 3 Oct 2008 | Bench test in Tampa of the Cornell ARI/QM software with a VDL 6000: `BMS` requests, `!VTBBM` sentences, `$AIABK` acks, mode-switch captures. |
| `20081005-capecod-visitor/` | 5 Oct 2008 | Swapping the VDL 6000 at the Cape Cod National Seashore visitor centre (Provincetown) for one with Blue-Force firmware; receiver logs before/after and the Windows ARI logs/config. |
| `20081015-boston-pog/` | 15 Oct 2008 | The Boston (Mystic River) transponder: `whale_update` script logs showing buoy status → `ZoneMsg 366:34` encoding, plus receiver logs. |

Notes:

* `20081013.log` is the only file in the whole collection with the full six
  USCG metadata fields (`s`, `d`, `T`, `x`, `r`, epoch). Read it with
  `errors="replace"`; it ends mid-line.
* Message 8 DAC 366 FI 34 is **not** the IMO Circ. 289 Area Notice (DAC 1
  FI 22) nor the later USCG DAC 366 FI 22; its layout is documented in the
  project's own notes (zone type 0 = "Right Whales NOT OBSERVED", 1 = observed,
  etc.). Treat the field layout in the tutorial as *reverse-engineered from
  these logs* and say so.
