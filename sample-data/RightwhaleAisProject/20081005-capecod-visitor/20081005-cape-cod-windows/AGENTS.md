# AGENTS.md — `sample-data/RightwhaleAisProject/20081005-capecod-visitor/20081005-cape-cod-windows/`

Files copied from the Windows PC running Cornell's **AIS Radio Interface
(ARI)** at the Provincetown ("Ptown") site. ARI pulls `BMS,…` requests from a
remote Queue Manager over TCP (`QM_TCPIP_IP = 99.178.167.126`, port 4501) and
feeds `!VTBBM` sentences to the transceiver on COM1 at 38 400 baud.

| File | Size | Contents |
|---|---|---|
| `config.txt` | 620 B | ARI configuration (`PAWS = Ptown`, QM address/port, serial vs TCP radio interface, `RADIO_SERIAL_BAUD = 38400`, `RADIO_SERIAL_COM = 1`). |
| `ARI-Ptown-2792008.log` | 629 KB | Uncompressed ARI log for day-of-year 279 (5 Oct 2008): every 5 min a batch of `BMS,SBNMS,43.000,-68.300,41.000,-71.300,<armoured payload>,<fill>*<hh>` lines received from the QM (here the payload is already 6-bit armoured, e.g. `Fr880a005cimnAPs=i92h0,2*35`), followed by the `!VTBBM` sentences sent to the radio. |
| `ARI-Ptown-2672008.log.bz2` … `ARI-Ptown-2782008.log.bz2` | 8–55 KB each (12 files) | The same log for days 267–278 (23 Sep–4 Oct 2008), bzip2-compressed. |
| `status.log.bz2` | 15 KB | ARI status log since 22 Aug 2008 (`Started AIS Radio Interface software…`). |
| `error.log.bz2` | 1 KB | ARI errors (`CTcpIpClient::GetLine encountered closed TCP/IP connection`). |
| `sample message.txt` | 851 B | Ten `!AIBBM,2,…,8,…` two-fragment broadcast binary message sentences plus one `!ECBBM,1,1,0,A,0,3vpG8N6Kw<3<P6P1=87l0@,4*06` — handy BBM syntax examples with talker ids `AI` and `EC`. |
| `sample messages.txt` | 609 B | Eight more `!VTBBM` fragments. |

The `BMS` line format changed between the Tampa test (`../../20081003-AIS/`,
payload as `0`/`1` text) and these logs (payload already armoured); both are
project-internal formats, not standards.
