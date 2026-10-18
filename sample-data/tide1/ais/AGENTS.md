# AGENTS.md — `sample-data/tide1/ais/`

816 daily AIS logs `ais-YYYY-MM-DD` from the `nhjel` receiver (Jackson
Estuarine Lab, Great Bay NH), 17 Feb 2007 – 10 May 2009, 34 MB total, plus:

* `README.txt` — "nhjel == New Hampshire, Jackson Estuarine Laboratory" and a
  note that the receiver power cord was replaced on 8 May 2009 after a
  couple of weeks of outage.
* `ttyS22007-02-18` — an early capture in the *oldest* logger format: a
  `# <epoch>` comment line before each sentence and no trailing fields.

Three logger formats appear over the two years:

| Period | Shape |
|---|---|
| Feb 2007 | `# START LOGGING…` banner with only `SPEED`/`PORT`, a `# <epoch>` line every 5 min (timeout marks), sentences **without** any suffix: `!AIVDM,1,1,,A,15NLiJ0u0lJsvwVHbFItpJ>B00Rf,0*57` |
| mid 2007 – 2009 | Full banner (`TIMEOUT`, `STATIONID: nhjel`, `DAEMON MODE: True`), sentences with `,rnhjel,<epoch>` suffix |
| 2009 | Same, plus `# MARK: <epoch>` lines |

Daily files are small (tens to a few hundred sentences). Highlights:

* Most days: message 1/3 from a few ships, two-fragment message 5.
* Message types rarely seen elsewhere appear *because of the receiver's
  corruption*, e.g. the survey counts "message 10/11/13/15/16/17" here — most
  of those are garbage; verify bit length before use.
* Genuine rarities worth checking: message 9 (SAR aircraft) 2008-11-27,
  message 11 (UTC response) 2008-08-21 (88 lines!), message 16 (assignment),
  message 17 (DGNSS) 2009-03-11, message 6 DAC 1 FI 32 (tidal window?)
  2007-04-04, message 8 DAC 1 FI 11 (IMO 236 met/hydro) 2007-09-12.

When a chapter needs a clean example, prefer `../../tide3/ais/` (same
software, better receiver) and use this directory for the "bad data" lesson.
