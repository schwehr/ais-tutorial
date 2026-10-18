# AGENTS.md — `sample-data/RightwhaleAisProject/20081015-boston-pog/`

15 October 2008 at the Boston-side transponder (station id
`cnsvdl6000-boston-mystic`, a CNS VDL 6000 on the Mystic River). Here the
whale-notice generator was CCOM's own Python `whale_update` script rather than
the Cornell ARI chain, so these logs show the **encoding side in detail**.

| File | Size | Contents |
|---|---|---|
| `whale-update-2008-10-15.log` | 247 KB | `whale_update` run log: each run dumps the buoy status dictionary parsed from Cornell's XML (`'TSS-AB10': {'n_good_dats': …, 'last_detect': …, 'lat_dd': 42.07896, 'long_dd': -69.848579, 'last_voltage': 17.291, …}`), then ten `buoy: BMS,SBNMS,43.000,-68.300,41.000,-71.300,<zone>,<msgid>,<status>,<unix time>,0101101110100010,<payload bits>` lines. The buoy **battery voltage and detection counts** are the kind of telemetry that the project later considered piggy-backing in AIS fields. |
| `20081015-pog-update.log` | 339 KB | Debug output of the encoder: buoy list, the parsed XML dict, `setfrombits ZoneMsg 366:34 - 1 0 15 15 10 10 [zone_type: 0 [Right Whales NOT OBSERVED] ]` / `ZoneCircular` object dumps (the field structure of DAC 366 FI 34), interleaved with 314 `!AIVDM` and 194 `!AIVDO` lines from the transponder. |
| `cnsvdl60002008-10-15` | 760 KB | `serial_logger.py` capture of the transponder port: 5 332 `!AIVDM`, 2 722 `!AIVDO` (`103OvqUP…`, MMSI 003669717), 466 `$AIALR`, 21 `$AITXT`; message 8 whale notices as received back from the other transponders (`85MwpViKf18k…`). |
| `log.2008-10-15` | 180 KB | A second receiver (SR162G, no station id) on the same day: 1 238 `!AIVDM`, 679 `$GPRMC`/`!AIVDO` pairs, 12 `$AITXT`. |

Use the `ZoneMsg` dumps in `20081015-pog-update.log` as the authority for
the DAC 366 FI 34 bit layout when writing the binary-message chapter.
