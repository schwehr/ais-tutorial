# AGENTS.md — `sample-data/sbnms/`

Stellwagen Bank National Marine Sanctuary (SBNMS) vessel-traffic work.

| Directory | Contents |
|---|---|
| `2006/` | `cleaned-notes.txt` (9.7 KB): a scratch log of SQL fix-ups (`UPDATE transit SET endpos=…`, `INSERT INTO transit (userid,startpos,endpos) …`) made while splitting mis-joined vessel *transits* in a 2006 AIS ship-traffic database. Shows the "track segmentation" post-processing problem; no NMEA. |
| `2008/data/` | Twelve files `YYYYMM-5.ais.unique` (Jan–Dec 2008, 45–240 KB each, 9 085 lines total): **unique AIS message 5** (static and voyage data) sentences seen by the Boston-area N-AIS receivers `r003669945/6/7`, one month per file, in the USCG format with `s`, `d`, `T`, `x`, `r` and epoch fields. Each line is a *single-sentence* message 5 (`!AIVDM,1,1,1,A,55Mq@Q02=Kjm…,2*5C`) — the collection tool re-joined the two fragments into one over-long sentence, so these lines exceed the 82-character NMEA limit. |

Teaching uses: a clean, moderately sized source of message 5 (ship name,
call sign, IMO number, dimensions, ETA, destination, draught, ship type), the
`@`-padded six-bit strings, and the "receiver re-assembled" oddity that
parsers should tolerate.
