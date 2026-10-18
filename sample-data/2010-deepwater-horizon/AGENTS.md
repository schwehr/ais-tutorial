# AGENTS.md — `sample-data/2010-deepwater-horizon/`

Non-NMEA side products from the **Deepwater Horizon / Mississippi Canyon 252**
oil-spill response (May–July 2010), when CCOM supported NOAA's ERMA common
operating picture with AIS-derived vessel tracking. There is **no raw AIS
here**; the directory documents what the *consumers* of AIS data looked like
and the identity problems (bad MMSIs, ambiguous names) you meet when you join
AIS to a vessel list. Use it in the tutorial's "how AIS is used" discussions.

| File | Size | What it is |
|---|---|---|
| `response-vessels.org` | 17 KB | Emacs org-mode notes and table matching ~150 response vessels (from the ICS-209 style `Vessels_050410.pdf`) to MMSI / call sign / IMO number using N-AIS and the ITU MARS database. The preamble lists real-world identity bugs: a survey vessel transmitting the Nauticast factory-default MMSI `1193046`, an "illegal" 7-digit MMSI `1092094`, duplicate names (`FORTE`, `WARRIOR`), USCG cutters in encrypted mode. |
| `20100504-vessel-info.pdf` | 101 KB | "Incident Resources by Area — Mississippi Canyon 252", printed 4 May 2010: the resource list that `response-vessels.org` was built from. |
| `20100702-nais-georss.xml` | 47 KB | NOAA GeoRSS 2.0 feed of 120 vessel positions from N-AIS (`<title>BROOKS MCCALL - 338257000: 0KTS @ 02 Jul 2010 17:03:20 GMT</title>`, `<gml:pos>lat lon</gml:pos>`). Shows the typical lossy "AIS → RSS" product: name, MMSI, speed, time, point. |
| `20100702-BuoyTracks.xml` | 149 KB | GeoRSS feed of **SLDMB** (Self-Locating Datum Marker Buoy) drifters: per buoy a latest `gml:Point` with `<SeaSurfaceTemp>` and a `gml:LineString` track. Not AIS; the buoys report via satellite. |
| `20100727-skimmers.xml` | 167 KB | Geoforce asset-tracking XML (`<assets>` / `<asset>` with name, id, last update, lat, lon, group) for 442 skimmers and other response assets. Again not AIS — a satellite tracker — included to contrast with AIS coverage. |
| `scripts/AISUser-gall-20100713.ini` | 1.2 KB | Configuration for the USCG **AISUser** client (see `scripts/AGENTS.md`). |

Why it matters for the tutorial:

* The MMSI problems in `response-vessels.org` are exactly the validation cases
  a decoder should flag (MMSI < 100 000 000, MID out of range, group/coast
  station formats). See the MMSI discussion in the message 1–3 chapter.
* The GeoRSS and asset feeds show the minimal "position + identity + time"
  tuple that almost every downstream user of AIS actually wants.
