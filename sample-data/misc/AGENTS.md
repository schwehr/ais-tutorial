# AGENTS.md — `sample-data/misc/`

One file: `rv-auk-boston-harbor-log.2008-08-20` (7.3 MB, 95 000 lines), a
`serial_logger.py` capture (station `auk`) from the **SR162G receiver on the
NOAA R/V *Auk***, 20 August 2008, 10:37–21:17 UTC. Despite the file name the
`$GPRMC` track stays at 42°11.97′N 70°43.22′W (the Scituate MA harbour where
the *Auk* is based) with SOG ≤ 2 kn — the boat did not go anywhere.

Contents: 38 386 `$GPRMC` / `!AIVDO` pairs (1 Hz; the `!AIVDO` is the
receiver's GPS re-encoded as a message 1 with MMSI 0), 18 089 `!AIVDM`
(10 400 message 1, 5 257 message 3, 1 530 message 4 from Boston-area base
stations, 868 multi-part message 5 fragments, **34 message 8** whale notices
`85MwpTiKf…` from MMSI 003669739), 176 `$AITXT`.

A good mid-sized file for: a full day of message 1/3 from the Massachusetts
Bay TSS, base-station message 4 at a 10 s cadence, and message 8 DAC 366
FI 34 right-whale notices in their 2008 (pre-IMO Circ. 289) form.
