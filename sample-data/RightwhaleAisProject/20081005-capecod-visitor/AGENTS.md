# AGENTS.md — `sample-data/RightwhaleAisProject/20081005-capecod-visitor/`

5 October 2008: at the Cape Cod National Seashore visitor centre
(Provincetown, "Ptown") the CNS VDL 6000 transceiver was replaced by one with
the "Blue Force" BIOS so that it would **send whale notices without sending
its own position reports**. `README.txt` (223 B) records this.

| File | Size | Contents |
|---|---|---|
| `README.txt` | 223 B | The note quoted above. |
| `20081005-capecod-visitor-center-basement` | 11 KB | `serial_logger.py` capture (station `capecod-before`) from an **SR162G in the basement** just before the swap: 122 `!AIVDM`, including the transceiver's own message 3 (`303OvqQP00Jv`30H4k21Vwvp0000`, MMSI 003669739) and its two-fragment message 5. Nice, small, clean file. |
| `2008-10-05-transponderswap-hyperterm.txt` | 25 KB | HyperTerminal capture on the transceiver port during the swap: 260 `!AIVDO`, 217 `!AIVDM`, 52 `$PCNSR`, 27 `$AIALR`. No timestamps. |
| `20081005-cape-cod-windows/` | | Files copied off the Windows PC that ran the Cornell ARI software (see its `AGENTS.md`). |
