# Appendix A — References and how we cite them

Throughout the tutorial, citations look like **[M.1371-5 §3.1 Table 48]**:
a short key in square brackets, followed by the section/table inside that
document. The keys resolve to the entries below. Many of these documents are
collected (with a BibTeX file, `specs/specs.bib`) in the companion
*AIS Open Manual* repository, <https://github.com/schwehr/ais-open-manual>;
the ITU, IMO, IALA and USCG documents are also free downloads from the
publishers' sites given here. IEC and NMEA standards are paywalled; we cite
them by clause so you can look them up if you have access, and we always
give a free alternative description where one exists.

## Primary protocol specifications

| Key | Document |
|---|---|
| **[M.1371-5]** | ITU-R Recommendation M.1371-5 (02/2014), *Technical characteristics for an automatic identification system using time division multiple access in the VHF maritime mobile frequency band*. Annex 2 = link layer (TDMA, communication states), Annex 5 = application-specific messages, Annex 8 = message formats (Tables 46–84). <https://www.itu.int/dms_pubrec/itu-r/rec/m/R-REC-M.1371-5-201402-S!!PDF-E.pdf>. Earlier editions: M.1371-0 (1998), -1 (2001), -2 (2006), -3 (2007), -4 (2010); the message tables changed between editions and this tutorial notes where. |
| **[M.585-9]** | ITU-R Recommendation M.585-9 (05/2022), *Assignment and use of identities in the maritime mobile service* — the MMSI format rules (ship, coast station, group, SAR aircraft, AtoN, AIS-SART/MOB/EPIRB). <https://www.itu.int/dms_pubrec/itu-r/rec/m/R-REC-M.585-9-202205-S!!PDF-E.pdf> |
| **[IEC 61162-1]** | IEC 61162-1:2024 (ed. 6), *Maritime navigation and radiocommunication equipment and systems — Digital interfaces — Part 1: Single talker and multiple listeners*. The international version of NMEA 0183: sentence structure (§7), checksum (§7.2.3), six-bit field conversion (§7.3.3), TAG blocks (§7.1.2 in ed. 4+), and the sentence formatters GGA, GSA, GSV, RMC, VDM, VDO, ABM, BBM, ABK, ALR, TXT, VSD, SSD (§8). <https://webstore.iec.ch/en/publication/81251> |
| **[NMEA 0183]** | NMEA 0183 *Standard for interfacing marine electronic devices*, v4.11 (2018) / v4.30 (2023). TAG blocks were introduced in v4.00 (2008) and revised in v4.10 (2012). <https://www.nmea.org/nmea-0183.html> |
| **[AIVDM]** | Eric S. Raymond (with Kurt Schwehr et al.), *AIVDM/AIVDO protocol decoding*, GPSD project, rev. 1.58 (2023). The best free description of the sentence layer, armouring, every message type and many ASMs. <https://gitlab.com/gpsd/gpsd/-/blob/master/www/AIVDM.adoc> |
| **[NMEA Revealed]** | Eric S. Raymond, *NMEA Revealed*, GPSD project (2022) — a free description of the common GPS sentences (GGA, GSA, GSV, RMC, …). <https://gpsd.gitlab.io/gpsd/NMEA.html> |
| **[Toils]** | Eric S. Raymond and Kurt Schwehr, *The Toils of AIS: Why standardizing AIS takes more than simply writing down ITU, IMO, and IEC documents*, v0.2 (2011). Catalogue of protocol ambiguities. <https://zenodo.org/records/17328241> |

## Application-specific (binary) messages

| Key | Document |
|---|---|
| **[Circ.236]** | IMO SN/Circ.236 (2004), *Guidance on the application of AIS binary messages* — the first international ASM set (DAC 1 FI 11 met/hydro, dangerous cargo, fairway closed, tidal window, extended static, persons on board, pseudo-AIS targets). Superseded by Circ.289 but still seen on the air. |
| **[Circ.289]** | IMO SN.1/Circ.289 (2010), *Guidance on the use of AIS application-specific messages* — DAC 1 FI 16–40 (area notice, met/hydro FI 31, environmental, route, text, weather, ETA/RTA, berthing, …). <https://www.navcen.uscg.gov/sites/default/files/pdf/IMO_SN1_Circ289_Guidance_on_use_of_AIS_ASM.pdf> |
| **[Circ.290]** | IMO SN.1/Circ.290 (2010), *Guidance for the presentation and display of AIS application-specific messages information*. |
| **[Seaway R41]** | St. Lawrence Seaway / USCG PAWSS, *AIS data messaging formats and specifications*, Rev. 4.1 (2020) — DAC 316 and 366 water level, wind, lock schedule messages and the ABM/BBM encapsulation. <https://greatlakes-seaway.com/wp-content/uploads/2020/05/Seaway-AIS-message-specifications-R41.pdf> |
| **[UNH RFC]** | Kurt Schwehr et al., *AIS binary message specification RFC: water level example* (UNH CCOM, 2006) — the design notes behind the tide-gauge experiments in `sample-data/tide1`. |
| **[RTCM SC121]** | Kurt Schwehr, *AIS binary messages, domain examples and a case for an XML message definition language*, RTCM paper 220-2007-SC121-013 (2007). |
| **[IHR 2010]** | Lee Alexander and Kurt Schwehr, *New standards for providing meteorological and hydrographic information via AIS application-specific messages*, International Hydrographic Review (2010). |
| **[AtoN form v4]** | USCG, *AIS Aid-to-Navigation specification sheet, form v4 — NOAA whale notices* (2010): the registration of the right-whale notice messages used in `sample-data/RightwhaleAisProject`. |

## USCG network, metadata and encoding guidance

| Key | Document |
|---|---|
| **[AISUser]** | USCG R&D Center, *AISUser (v1.3.0) user manual* (Oct 2008). §3.6.2.8 defines the three output formats (unchanged, NMEA 4.00 TAG block, proprietary USCG R&DC format). |
| **[AISMiner]** | USCG R&D Center, *AIS Miner user manual* (Dec 2006). |
| **[AISSource.ini]** | USCG, *AISSource.ini INI validation description* (2009). |
| **[Encoding Guide]** | USCG Navigation Center, *U.S. AIS encoding guide: operational guidance for encoding shipborne AIS parameters* (2019) — what mariners are supposed to type into each static/voyage field. <https://www.navcen.uscg.gov/sites/default/files/pdf/AIS/USCG_Encoding_Guide.pdf> |
| **[Inspection]** | USCG, *AIS inspection checklist* (2019). |
| **[Safety Alerts]** | USCG, *Compilation of safety alerts and MSIBs regarding AIS* (2024) — Safety Alerts 1-05, 5-05, 10-08, 05-10, 7-10, 13-18 and 01-24 on wrong MMSIs, inaccurate static data and civil penalties. (The separate USCG MSIBs on non-type-approved fishing-gear AIS devices are not part of this compilation.) |
| **[EAIS IDD]** | USCG, *Encrypted AIS (EAIS) interface design description* (2016) — the DAC 366 FI 56/57 encrypted position/static messages seen in `sample-data/tide3`. Not publicly available; we describe only what is observable. |
| **[33 CFR 164.46]** | US carriage requirement for AIS. <https://www.ecfr.gov/current/title-33/chapter-I/subchapter-P/part-164/section-164.46> |

## IMO / IALA / IEC equipment standards (for context)

| Key | Document |
|---|---|
| **[MSC.74(69)]** | IMO Resolution MSC.74(69) Annex 3 (1998), *Performance standards for a universal shipborne AIS* — reporting intervals, information content. |
| **[A.1117(30)]** | IMO Resolution A.1117(30) (2017), *IMO ship identification number scheme* — the seven-digit IMO number and its check digit (weights 7…2, sum modulo 10). |
| **[A.1106(29)]** | IMO Resolution A.1106(29) (2015), *Revised guidelines for the onboard operational use of shipborne AIS*. |
| **[MSC.140(76)]** | IMO Resolution MSC.140(76) (2002), *Protection of the AIS VHF data link*. |
| **[SN/Circ.227]** | IMO SN/Circ.227 (2003), *Guidelines for the installation of a shipborne AIS* (antenna/dimension reference points A, B, C, D). |
| **[SN/Circ.244]** | IMO SN/Circ.244 (2005), *Guidance on the use of the UN/LOCODE in the destination field*. |
| **[IALA G1082]** | IALA Guideline G1082 ed. 2 (2016), *An overview of AIS*. <https://www.navcen.uscg.gov/sites/default/files/pdf/IALA_Guideline_1082_An_Overview_of_AIS.pdf> |
| **[IALA G1062]** | IALA Guideline G1062 (2008), *On the establishment of AIS as an aid to navigation*. |
| **[IALA R0126]** | IALA Recommendation R0126 (A-126) ed. 2 (2019), *The use of AIS in marine aids to navigation services*. |
| **[IALA G1098]** | IALA Guideline G1098 ed. 1 (2013), *The application of AIS-AtoN on buoys* — §3.5.2 on status-bit monitoring (battery, light, RACON). |
| **[IALA R0143]** | IALA Recommendation R0143 (O-143) ed. 1.2 (2013), *On the provision of virtual aids to navigation*. |
| **[IEC 61993-2]** | IEC 61993-2:2018, Class A shipborne AIS equipment. |
| **[IEC 62287-1/-2]** | IEC 62287-1:2017 (Class B CS) and 62287-2:2017 (Class B SO). |
| **[IEC 62320-1/-2/-3]** | IEC 62320-1:2015 (base stations), -2:2016 (AtoN stations), -3:2015 (repeaters). |
| **[M.1084]** | ITU-R Recommendation M.1084-5 (03/2012), *Interim solutions for improved efficiency in the use of the band 156–174 MHz by stations in the maritime mobile service* — Annex 4: channel numbering including the four-digit simplex-use-of-duplex numbers (2087/2088). <https://www.itu.int/rec/R-REC-M.1084> |
| **[M.823]** | ITU-R Recommendation M.823-3 (03/2006), *Technical characteristics of differential transmissions for GNSS from maritime radio beacons* — the RTCM SC-104 v2 message/health format referenced by M.1371-5 for message 17. <https://www.itu.int/rec/R-REC-M.823> |
| **[M.2092]** | ITU-R M.2092-0 (2015), VDES — where AIS is heading. |
| **[M.2169]** | ITU-R Report M.2169 (2009), *Improved satellite detection of AIS*. <https://www.itu.int/pub/R-REP-M.2169> |

## Data provenance

All sample sentences are excerpts from logs recorded 2006–2011 by Kurt
Schwehr and colleagues (UNH CCOM, NOAA SBNMS, Cornell BRP) as described in
`sample-data/AGENTS.md` and the per-directory guides there. Each chapter
names the source file and line numbers of every excerpt it uses.
