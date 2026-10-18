# AGENTS.md — `sample-data/RightwhaleAisProject/20081003-AIS/`

3 October 2008 bench test (Tampa, FL) of the Cornell **AIS Radio Interface
(ARI)** and **Queue Manager (QM)** Windows programs talking to a **CNS Systems
VDL 6000** Class A transponder, before the equipment went to Cape Cod. All
timestamps in the Windows logs are local `MM/DD/YY HH:MM:SS`; line endings
`\r\r\n`.

| File | Size | Contents |
|---|---|---|
| `QM-Cornell-Tampa-2772008.log` | 158 KB | Queue Manager log (day-of-year 277). Shows the **upstream request format** `Received from FF: BMS,SBNMS,43.000,-68.300,41.000,-71.300,<zone>,<msgid>,<…>,<unix time>,0101101110100010,<payload bits>` — a bounding box, then the 16-bit DAC/FI header `0101101110 100010` (366/34) and the raw binary payload as `0`/`1` text — followed by `Adding new message to queue` / `Dequeing message` lines. |
| `ARI-Tampa-2772008.log` | 30 KB | Radio Interface log: connection retries, then the **`!VTBBM` sentences** built from the queue (`!VTBBM,2,1,0,0,8,Fr880KRP5cim…,0*06` / `!VTBBM,2,2,0,0,8,…`) — two-fragment broadcast binary message requests, channel `0` (= no preference), message type `8`. `VT` is the talker id the software chose. |
| `whale.txt` | 268 B | Four `!VTBBM` lines (two 2-fragment messages) ready to paste into a terminal. |
| `whale_working.txt` | 46 B | A single short `!VTBBM,1,1,0,0,8,Fr882=7h5cimnAPs=i92h0,2*54` that was confirmed to transmit. |
| `AIS_whale` | 83 KB | HyperTerminal capture of the transponder port while sending: 1 304 `!AIVDO` (own position `103OvrpP…`, MMSI 003669739), 13 `!VTBBM` echoed, **13 `$AIABK,,,8,0,3`** acknowledgements (type `3` = "requested broadcast of message 8 has been successfully completed", IEC 61162-1 ABK table), 261 `$PCNSR`, 176 `$AIALR`. |
| `AIS_modes`, `AIS_modes1` | 3 KB, 8 KB | Captures while cycling the `$PCNSS,BFP,…` Blue-Force transmit modes; `$PCNSR` responses, `!AIVDO`, `$AIALR`. |
| `AIS_ON_OFF` | 4 KB | `!AIVDO` lines around power cycling; the "no position" report `103OvrpP?w<tSF0l4Q@>4?wv0000` (lon 181°, lat 91°). |
| `standard.txt`, `silent.txt`, `secure.txt` | ≤ 40 B | The three `$PCNSS,BFP,…` mode commands (same as `../cns-*-dos.txt`). |
| `ccom-sr162g2008-10-03.vdm` | 31 KB | SR162G receiver at CCOM the same day (station `ccom-sr162g`): 310 `!AIVDM`, 6 `$AITXT`, and the receiver's boot banner `RX YACHT AIS Receiver v1.3`, `RECE FRQ R1F1619750 / R2F1620250` (161.975 / 162.025 MHz). |
| `ais_error.log`, `que_error.log` | 1 KB, 103 B | ARI/QM error logs (`Can't connect to IP localhost`, `An error occured opening com port`, `Missing EOL while reading`). |

Teaching uses: the complete **send path** for a binary message (application →
`BBM` sentence → transponder → `ABK` → on-air `VDM` seen by another receiver),
BBM fragmenting and fill bits, and the 6-bit armouring of a payload that
starts with DAC/FI bits.
