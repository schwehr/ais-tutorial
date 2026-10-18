# AGENTS.md — `sample-data/2010-deepwater-horizon/scripts/`

One file: `AISUser-gall-20100713.ini`, the INI configuration of the
**USCG R&D Center "AISUser"** client program (v1.x, 2008–2010) as used on a
CCOM machine (`Client.Name=1142_09_UNH_NOAA_ERMA_JHC_2`) on 13 July 2010 to
pull the national N-AIS feed for the Deepwater Horizon response.

Why it is interesting for the tutorial's "USCG metadata" and "TAG block"
chapters:

* `Server.Connection.1.Name=nais.uscg.mil` / `Port=31415` and the disaster
  recovery mirror `naisdr.uscg.mil` — the upstream feed addresses.
* `Monitor.Connection.1.OutputFormat=2` and `Monitor.Connection.2.OutputFormat=0`:
  AISUser can re-emit the feed on local TCP ports (31414, 31415) in three
  formats — `0` unchanged, `1` NMEA 4.0 **TAG block** format, `2` the
  proprietary **USCG R&D Center** comma-appended format
  (`…*hh,s1234,d-107,T39.92,x150977,r003669946,1223948560`). The tutorial's
  USCG-format samples came out of option 2 style feeds.
* `NMEA0183.TAG.Block.Version=4.00` — the TAG block version AISUser emits.
* `Server.Connection.n.CompressionType=1` — the wire feed was compressed.

Line endings are `\r\r\n`. Do not edit.
