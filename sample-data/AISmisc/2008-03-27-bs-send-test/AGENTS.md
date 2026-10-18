# AGENTS.md — `sample-data/AISmisc/2008-03-27-bs-send-test/`

27 March 2008, CCOM room 159: transmit tests with an **L-3 AIS base station**
(station ids `bs1`, `bs2`) while an SR162 receiver (`ccom159-sr162`) listened
on the bench. All files are `serial_logger.py` captures; the `p2`/`p3` files
are the same base-station ports captured again later in the day and contain
stray binary (`\x00\x00\x00$PL3COMFLASH,VER:02.3,…` firmware banners).

| File | Size | Station | Contents |
|---|---|---|---|
| `bs1-2008-03-27` | 6 KB | `bs1` | 28 `!BSVDO` **message 4** (base station report, `4h0C5GQuPun9n<tSF0l4Q@001`;f`), 47 `$BSALR` (`Tx malfunction`, `Antenna VSWR exceeds limit`, `Rx channel 1 malfunction` — no antenna attached), 5 `$BSADS,312670-BS,220947.00,A,3,N,N` (base-station "device status" sentence with its id and UTC). |
| `bs2-2008-03-27` | 1 KB | `bs2` | 5 `!BSVDO`, 3 `$BSALR`, 1 `$BSADS`. |
| `p2-2008-03-27`, `p3-2008-03-27` | 1.5 KB, 3.4 KB | `p2`, `p3` | Later captures of the same ports, mostly bootloader garbage, a few `!BSVDO`/`$BSALR`. |
| `sr162-2008-03-27` | 31 KB | `ccom159-sr162` | What the receiver heard: 388 `!AIVDM` **all message 4** from the base station (`4h0C5GQuPumHB<tSF0l4Q@001t00`), 5 `$AITXT`. |

Teaching uses: message 4 with a station that has **no GNSS fix** (position
181°E/91°N "not available", EPFD type and UTC fields), the `BS` talker id, and
comparing `!BSVDO` (sent) with `!AIVDM` (received).
