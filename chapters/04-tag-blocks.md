# 4. NMEA TAG blocks

> **What you will learn**
> * Why NMEA 0183 v4.00 added a second, backslash-delimited "TAG block"
>   layer in front of sentences, and what the seven parameter codes
>   `c d g n r s t` carry.
> * How the TAG block checksum is computed (exactly like a sentence
>   checksum) and how to verify one by hand.
> * How the `g:` grouping parameter solves the fragment-interleaving
>   problem that chapter 2's `Assembler` could only paper over.
> * How `tagblock.py` and its C++/Rust/TypeScript twins split the block off
>   a line *before* NMEA parsing, and how `decode.*` surfaces it as a
>   `"tagblock"` sub-object.
> * Where TAG blocks come from in practice — including the honest fact that
>   none of this tutorial's recordings contain one, and how the chapter's
>   sample file was manufactured from USCG-format data.

## 4.1 Where this fits

Chapter 1 introduced the NMEA 0183 sentence: `$` or `!`, an address field,
comma-separated fields, `*hh` checksum, `CR LF`, at most 82 characters
[IEC 61162-1 §7.2]. That container has no room for *metadata about the
sentence itself*: when was it received, by which of a network's hundred
receivers, and which other sentences belong with it. Every organisation
that built an AIS network solved this in its own way — the US Coast Guard
by appending fields *after* the checksum (chapter 5), others by prefixing a
timestamp, others with out-of-band framing.

NMEA 0183 **v4.00 (2008)** standardised one answer: the **TAG block**, a
backslash-delimited group of `code:value` parameters placed *in front of*
the sentence on the same line [NMEA 0183 §7 "TAG blocks"]. **v4.10 (2012)**
revised the parameter list and the grouping rules into the form used today
[NMEA 0183 v4.10 §7]. The international twin of NMEA 0183, IEC 61162-1,
carries the same text as §7.1.2 "TAG block" from edition 4 (2010) onwards
[IEC 61162-1 §7.1.2]. GPSD's documentation has a shorter, free description
[AIVDM "NMEA Tag Blocks"].

### Operational use

On a ship's bridge TAG blocks are rare: the equipment is wired
point-to-point, so "who sent this" is already known. The block earns its
keep in **networked** installations:

* **Shore receiver networks** (vessel traffic services (VTS), national AIS networks such as the US
  N-AIS, commercial aggregators). A single TCP stream carries the output of
  many receivers; `s:` says which one heard the message and `c:` says when,
  so that downstream consumers can de-duplicate and time-order.
* **Ethernet marine networks** (IEC 61162-450, "LWE"). Sentences travel in
  UDP datagrams; the TAG block supplies the source identification and the
  line counter `n:` that lets a receiver detect lost datagrams
  [IEC 61162-1 §7.1.2].
* **Loggers and replayers.** A log with `c:` timestamps can be replayed at
  the original pace; a log with `g:` groups can be shuffled or filtered
  without breaking multi-sentence messages.
* **USCG AISUser clients.** The program that distributed the N-AIS feed to
  researchers (chapter 5) could re-emit the feed in TAG block format on
  request (`OutputFormat=1`) [AISUser §3.6.2.8]; the INI file in
  `sample-data/2010-deepwater-horizon/scripts/` has
  `NMEA0183.TAG.Block.Version=4.00` for exactly that path.

### Unattended / automated uses

Machines use TAG blocks for three things that people never look at:

1. **Loss detection.** A monotonically increasing `n:` per source lets a
   consumer count gaps. In a satellite or cellular back-haul a jump of 50
   in `n:` between two consecutive lines is a quantified outage, not a
   vague "the feed looked thin".
2. **Reassembly across interleaving.** Chapter 2's `Assembler` keys
   fragments on (talker, channel, sequential message id). The sequential
   id is a single digit 0–9 [IEC 61162-1 §8 VDM], so when a server merges
   twenty receivers, fragment `2,2,6` from receiver A is indistinguishable
   from fragment `2,2,6` from receiver B unless something else tells them
   apart. The `g:` triple `n-total-id` with a server-assigned `id` does
   exactly that, and `s:` adds a second key.
3. **Routing.** `d:` (destination) and `s:` (source) are filter keys: a
   gateway can forward only blocks whose `d:` matches it. The standard also
   defines query/configuration sentences by which a listener can ask a
   talker to change TAG block behaviour (which parameters to send, the time
   unit of `c:`) [NMEA 0183 v4.10 §7]. Those sentences are out of scope
   here; we only *read* TAG blocks.

## 4.2 The specification

### 4.2.1 Syntax

```text
\s:003669947,c:1223901985,n:4*4E\!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02
^                            ^  ^^
|                            |  |+-- closing backslash; the sentence follows at once
|                            |  +--- two hex digits: XOR of every character between the backslashes, before the '*'
|                            +------ '*' separates parameters from the checksum
+----------------------------------- opening backslash
```

| Element | Rule | Source |
|---|---|---|
| Delimiters | `\` (0x5C) opens and closes the block. The closing `\` is immediately followed by the sentence's `$`/`!`, or by `CR LF` if the block stands alone. | [NMEA 0183 v4.10 §7], [IEC 61162-1 §7.1.2] |
| Parameters | One or more `code:value` items separated by `,`. Codes are single, case-sensitive letters. A value may not contain `\`, `*` or `,`. | [NMEA 0183 v4.10 §7] |
| Checksum | `*hh` after the last parameter: the 8-bit XOR of all characters between the opening `\` and the `*`, written as two upper-case hex digits — the same algorithm as the sentence checksum [IEC 61162-1 §7.2.3]. | [NMEA 0183 v4.10 §7] |
| Length | The whole block, delimiters included, is limited to **80 characters**; the sentence after it keeps its own 82-character limit. | [NMEA 0183 v4.10 §7], [IEC 61162-1 §7.1.2] |
| Position | Precedes the sentence on the same line. One block per line. | [NMEA 0183 v4.10 §7] |

### 4.2.2 Parameter codes

| Code | Name | Type and limits | Meaning in practice | Source |
|---|---|---|---|---|
| `c` | UNIX time | Unsigned integer. Seconds since 1970-01-01 00:00 UTC by default; **milliseconds** if the talker was configured for it. | Time the *talker* (receiver, server) emitted or received the sentence — not the time inside the AIS message. | [NMEA 0183 v4.10 §7], [AIVDM "NMEA Tag Blocks"] |
| `d` | Destination identification | String, ≤ 15 characters. | Who the sentence is addressed to on a network; used for filtering. | [NMEA 0183 v4.10 §7] |
| `g` | Sentence grouping | Three unsigned integers `n-total-id`: this is sentence *n* of *total* in group *id*. | Ties the sentences of a multi-sentence message together even when other sentences are interleaved. The block with `g:` applies to the group, not just the line. | [NMEA 0183 v4.10 §7], [AIVDM "NMEA Tag Blocks"] |
| `n` | Line count | Unsigned integer, incremented per line by the talker. | Gap detection; also a cheap unique key per source. | [NMEA 0183 v4.10 §7] |
| `r` | Relative time | Unsigned integer. | Time relative to a reference the talker established (via the configuration sentences). Almost never seen in the wild; the unit is whatever was configured. | [NMEA 0183 v4.10 §7], [AIVDM "NMEA Tag Blocks"] |
| `s` | Source identification | String, ≤ 15 characters. | The talker's identity: a receiver, a station id, a server. | [NMEA 0183 v4.10 §7] |
| `t` | Text string | String, ≤ 15 characters. | Free text. The only place a talker can put anything the standard did not foresee. | [NMEA 0183 v4.10 §7] |

A few consequences that matter when you parse:

* **`c:` seconds vs milliseconds.** The standard allows either, and the
  block does not say which. A practical rule: 10<sup>10</sup> seconds is
  the year 2286, so any `c:` larger than that is milliseconds; anything
  smaller is seconds. The rule fails only for a millisecond talker emitting
  times before 1970-04-26 (10<sup>10</sup> ms), which does not happen.
  Our `TagBlock.time` property returns the raw integer and leaves the unit
  decision to the caller (exercise 4.7.1).
* **No private codes.** NMEA 4.10 does not provide an "experimental" or
  "proprietary" parameter code. Anything that is not time, source,
  destination, group or line count must go into `t:` — and `t:` is capped
  at 15 characters. We will feel that limit in §4.2.4.
* **The block and the sentence are checksummed separately.** A corrupted
  block does not invalidate the sentence, and vice versa. Our output
  therefore reports `tagblock.checksum_ok` separately from the sentence's
  `error: checksum mismatch`.
* **`g:` is the only parameter that spans lines.** Every other parameter
  describes the one sentence that follows. With `g:`, the standard says the
  block's parameters apply to the whole group — so a `c:` on the *first*
  fragment of a type 5 is the timestamp of the message, even if later
  fragments carry their own blocks with slightly later `c:` values.

### 4.2.3 Why `g:` exists — the interleaving problem

The VDM sentence carries its own fragment bookkeeping: `total`, `number`
and a one-digit sequential message id [IEC 61162-1 §8 VDM]. Chapter 2 built
an `Assembler` around those three fields. The design assumes one talker. A
server that merges many receivers breaks it in two ways:

1. Two receivers hear the *same* two-fragment type 5 and both forward it
   with sequential id `6`. The merged stream now has `2,1,6 2,1,6 2,2,6 2,2,6`
   and the assembler pairs the wrong halves — silently, because the halves
   are identical, or noisily when the ships differ.
2. A receiver's sequential id wraps every ten multi-sentence messages, so
   in a busy stream fragments from *different* messages with the same id
   can be a fraction of a second apart.

`g:n-total-id` fixes both because `id` is assigned by the merging talker
and can be as large as needed (the GPSD example uses 73874
[AIVDM "NMEA Tag Blocks"]), and `s:` tells you which receiver each fragment
came from. A parser that trusts TAG blocks keys its assembler on
(`s`, `g.id`) instead of (talker, channel, sequential id).

### 4.2.4 Where our sample data came from

**No TAG blocks occur in any raw recording used by this tutorial.** The
recordings span 2006–2011; NMEA 4.00 was published in late 2008, v4.10 in
2012, and receiver firmware and network software took years to follow. The
USCG feed that most of chapter 5's data comes from used the pre-standard
suffix format throughout that period.

So the file `data/ch04-tag-blocks.nmea` is **derived**: it was produced
from `data/ch05-uscg-metadata.nmea` by the converter
`code/python/tools/uscg_to_tagblock.py`, which does what the USCG's
AISUser client did when a user switched a monitor port from
`OutputFormat=2` (USCG R&D Center suffix format) to `OutputFormat=1`
(NMEA 4.00 TAG block) [AISUser §3.6.2.8]. The mapping is:

| USCG suffix field (chapter 5) | TAG block parameter | Note |
|---|---|---|
| `r003669946` (receiving station) | `s:003669946` | the `r` prefix is dropped |
| `1223901985` (logger UNIX time) | `c:1223901985` | fractional times are truncated to whole seconds |
| `d-074` (dBm), `T25.49701619` (time of arrival) | `t:d-74 T25.50` | **no standard code** — packed into free text, cut to 15 characters |
| `s34486` (RSSI), `x78124` (receiver counter) | *(dropped)* | would not fit in `t:` |
| fragment `2,1,6` / `2,2,6` of the same receiver | `g:1-2-1` / `g:2-2-1` | group id allocated per (talker, channel, sequential id) |
| — | `n:1`, `n:2`, … | line counter added by the converter |

The conversion is **lossy**: the RSSI, the receiver's own sentence
counter and six of the eight decimals of the arrival time are gone. That
is not a flaw of the converter; it is the honest consequence of a
15-character free-text field being the only extension point. Chapter 5
explains what those lost fields are good for.

The converter itself is short enough to read in full:

```python
def convert(line: str, counter: int, group_ids: dict[tuple[str, str, str], int]) -> str:
    s = parse_sentence(line)
    meta = parse_uscg_fields(s.after_checksum)
    params: dict[str, str] = {}
    if "station" in meta:
        params["s"] = str(meta["station"])
    if "time" in meta:
        params["c"] = str(int(float(str(meta["time"]))))
    extras: list[str] = []
    if "signal_dbm" in meta:
        extras.append(f"d{meta['signal_dbm']}")
    if "time_of_arrival_s" in meta:
        extras.append(f"T{float(str(meta['time_of_arrival_s'])):.2f}")
    if extras:
        params["t"] = " ".join(extras)[:15]
    if s.formatter in ("VDM", "VDO") and s.fields[0] != "1":
        total, number, seq = s.fields[0], s.fields[1], s.fields[2]
        key = (s.talker, s.fields[3], seq)
        if number == "1" or key not in group_ids:
            group_ids[key] = len(group_ids) + 1
        params["g"] = f"{number}-{total}-{group_ids[key]}"
    params["n"] = str(counter)
    # Keep only the sentence proper (drop the USCG suffix after the checksum).
    body = s.raw
    star = body.find("*")
    sentence = body[: star + 3] if star >= 0 else body
    return format_tag_block(params) + sentence
```

`parse_sentence` is chapter 1's, `parse_uscg_fields` is chapter 5's, and
`format_tag_block` is in §4.4. Running
`python3 code/python/tools/uscg_to_tagblock.py < data/ch05-uscg-metadata.nmea`
reproduces `data/ch04-tag-blocks.nmea` byte for byte.

## 4.3 Decoding one TAG block by hand

Take line 4 of `data/ch04-tag-blocks.nmea` (its ancestor is line 4 of
`data/ch05-uscg-metadata.nmea`, i.e. line 7 of
`sample-data/RightwhaleAisProject/20081013.log`):

```text
\s:003669947,c:1223901985,n:4*4E\!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02
```

**Step 1 — find the block.** The line starts with `\`; the next `\` is at
offset 32. Everything between is the block interior:
`s:003669947,c:1223901985,n:4*4E`.

**Step 2 — split off the checksum.** Partition at `*`: body
`s:003669947,c:1223901985,n:4`, given checksum `4E`.

**Step 3 — XOR the body.** Start at 0 and XOR each character's ASCII code.
The running value after each character:

```text
 s   :   0   0   3   6   6   9   9   4   7   ,   c   :
73  49  79  49  7A  4C  7A  43  7A  4E  79  55  36  0C
 1   2   2   3   9   0   1   9   8   5   ,   n   :   4
3D  0F  3D  0E  37  07  36  0F  37  02  2E  40  7A  4E
```

The final value is `0x4E`, matching the `*4E` in the block: `checksum_ok`
is true. (The sentence's own `*02` is checked separately by chapter 1's
code.)

**Step 4 — split the parameters.** Split the body on `,` and each item on
the first `:`:

| item | code | value | typed view |
|---|---|---|---|
| `s:003669947` | `s` | `003669947` | `source` = `"003669947"` |
| `c:1223901985` | `c` | `1223901985` | `time` = 1223901985 — 2008-10-13 12:46:25 UTC (less than 10<sup>10</sup>, so seconds) |
| `n:4` | `n` | `4` | fourth line from this talker |

No `g:`, so the block applies to the single sentence that follows, a type 1
position report from maritime mobile service identity (MMSI) 538002219
(chapter 3).

## 4.4 The code

The module is deliberately tiny. It has one data type, `TagBlock`, holding
the raw parameter dictionary plus the checksum verdict, with *typed views*
(`time`, `source`, `group`) computed on demand; one function
`split_tag_block(line)` that returns the block (or nothing) and the
remainder of the line; and one function `format_tag_block(params)` used by
the converter. The driver `decode.*` calls `split_tag_block` **first**,
before `parse_sentence`, because chapter 1's parser would otherwise treat
the backslash text as garbage in front of the `!`.

### Python

```python
@dataclass
class TagBlock:
    params: dict[str, str] = field(default_factory=dict)
    checksum_ok: bool = False
    raw: str = ""

    # Typed views of the common parameters. Missing -> None.
    @property
    def time(self) -> int | None:
        c = self.params.get("c")
        return int(c) if c and c.isdigit() else None

    @property
    def source(self) -> str | None:
        return self.params.get("s")

    @property
    def group(self) -> tuple[int, int, int] | None:
        """(sentence number, total sentences, group id) from the g: parameter."""
        g = self.params.get("g")
        if not g:
            return None
        parts = g.split("-")
        if len(parts) != 3 or not all(p.isdigit() for p in parts):
            return None
        return int(parts[0]), int(parts[1]), int(parts[2])

    def to_dict(self) -> dict:
        d: dict = {"params": dict(self.params), "checksum_ok": self.checksum_ok}
        if self.time is not None:
            d["time"] = self.time
        if self.group is not None:
            d["group"] = list(self.group)
        return d


def split_tag_block(line: str) -> tuple[TagBlock | None, str]:
    line = line.strip()
    if not line.startswith("\\"):
        return None, line
    end = line.find("\\", 1)
    if end < 0:
        raise ValueError("unterminated TAG block")
    inner = line[1:end]
    body, star, hexpart = inner.partition("*")
    ok = False
    if star and len(hexpart) >= 2:
        try:
            ok = int(hexpart[:2], 16) == checksum(body)
        except ValueError:
            ok = False
    params: dict[str, str] = {}
    for item in body.split(","):
        code, colon, value = item.partition(":")
        if colon:
            params[code] = value
    return TagBlock(params, ok, inner), line[end + 1:]


def format_tag_block(params: dict[str, str]) -> str:
    body = ",".join(f"{k}:{v}" for k, v in params.items())
    return f"\\{body}*{checksum(body):02X}\\"
```

And the first thing `decode.py`'s `process_line` does with a non-comment
line:

```python
    try:
        tag, rest = split_tag_block(stripped)
    except ValueError as e:
        out["error"] = str(e)
        out["raw"] = stripped
        return out
    if tag is not None:
        out["tagblock"] = tag.to_dict()

    try:
        s = parse_sentence(rest)
    ...
```

> **Language notes (Python)**
> * `str.partition`, `@property` and `dict.get` are the chapter 1 constructs
>   again: `partition` is used twice here (once for `*`, once for `:`), and
>   testing the middle element (`if colon:`) is the idiomatic "was the
>   separator there?" check; the `@property` views are *derived* from
>   `params` on every access, so they can never go stale.
> * `"\\"` in Python source is a one-character string containing a
>   backslash; the doubling is source syntax, not data.

### C++

```cpp
struct TagBlock {
    std::map<std::string, std::string> params;
    bool checksum_ok = false;
    std::string raw;

    std::optional<long long> time() const {
        auto it = params.find("c");
        if (it == params.end() || !py::isdigit(it->second)) return std::nullopt;
        return py::to_int(it->second);
    }

    std::optional<std::string> source() const {
        auto it = params.find("s");
        if (it == params.end()) return std::nullopt;
        return it->second;
    }

    std::optional<std::array<long long, 3>> group() const {
        auto it = params.find("g");
        if (it == params.end() || it->second.empty()) return std::nullopt;
        std::vector<std::string> parts = py::split(it->second, '-');
        if (parts.size() != 3) return std::nullopt;
        std::array<long long, 3> out{};
        for (std::size_t i = 0; i < 3; ++i) {
            if (!py::isdigit(parts[i])) return std::nullopt;
            std::optional<long long> v = py::to_int(parts[i]);
            if (!v) return std::nullopt;
            out[i] = *v;
        }
        return out;
    }

    Json to_dict() const {
        Json::Object p;
        for (const auto& [code, value] : params) p[code] = value;
        Json d = Json::Object{{"params", p}, {"checksum_ok", checksum_ok}};
        if (auto t = time()) d["time"] = *t;
        if (auto g = group()) d["group"] = Json::Array{(*g)[0], (*g)[1], (*g)[2]};
        return d;
    }
};

inline std::pair<std::optional<TagBlock>, std::string> split_tag_block(std::string_view input) {
    std::string line = py::strip(input);
    if (!py::startswith(line, "\\")) return {std::nullopt, line};
    std::size_t end = line.find('\\', 1);
    if (end == std::string::npos) throw std::invalid_argument("unterminated TAG block");
    std::string inner = line.substr(1, end - 1);
    py::Partition at_star = py::partition(inner, '*');
    bool ok = false;
    if (at_star.found && at_star.tail.size() >= 2) {
        std::optional<long long> given = py::to_int(py::slice(at_star.tail, 0, 2), 16);
        ok = given.has_value() && *given == checksum(at_star.head);
    }
    TagBlock tag;
    for (const std::string& item : py::split(at_star.head, ',')) {
        py::Partition kv = py::partition(item, ':');
        if (kv.found) tag.params[kv.head] = kv.tail;
    }
    tag.checksum_ok = ok;
    tag.raw = inner;
    return {tag, line.substr(end + 1)};
}
```

> **Language notes (C++)**
> * `std::map<std::string, std::string>` is the dictionary of chapter 1,
>   always sorted by key — which is why `to_dict()` can copy it straight
>   into the JSON object. Look up with `find`/`end()`, never with
>   `params[key]`, which would silently *insert* an empty string.
> * `std::optional<T>` chains: each typed view returns `std::nullopt` on any
>   failure, and the caller tests it with `if (auto t = time())` — the
>   declaration inside the `if` is in scope only when the optional holds a
>   value. `*t` reads the value.
> * `py::partition` is the tutorial's own helper (in `pycompat.hpp`)
>   mirroring Python's `str.partition`; it returns a small struct
>   `{head, found, tail}` because C++ has no multiple return values other
>   than `std::pair`/`std::tuple`.
> * `std::array<long long, 3>` is a fixed-size array that can be returned by
>   value — the natural translation of Python's three-tuple.
> * The function throws `std::invalid_argument` for the one unrecoverable
>   case; `decode.cpp` catches it and emits the same error text as Python.

### Rust

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagBlock {
    pub params: BTreeMap<String, String>,
    pub checksum_ok: bool,
    pub raw: String,
}

impl TagBlock {
    pub fn time(&self) -> Option<i64> {
        let c = self.params.get("c")?;
        if is_ascii_digits(c) {
            c.parse().ok()
        } else {
            None
        }
    }

    pub fn source(&self) -> Option<&str> {
        self.params.get("s").map(String::as_str)
    }

    pub fn group(&self) -> Option<(i64, i64, i64)> {
        let g = self.params.get("g")?;
        if g.is_empty() {
            return None;
        }
        let parts: Vec<&str> = g.split('-').collect();
        if parts.len() != 3 || !parts.iter().all(|p| is_ascii_digits(p)) {
            return None;
        }
        Some((parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?))
    }

    pub fn to_json(&self) -> Json {
        let mut params = Obj::new();
        for (k, v) in &self.params {
            put(&mut params, k, v.as_str());
        }
        let mut d = Obj::new();
        put(&mut d, "params", params);
        put(&mut d, "checksum_ok", self.checksum_ok);
        if let Some(t) = self.time() {
            put(&mut d, "time", t);
        }
        if let Some((n, total, id)) = self.group() {
            put(&mut d, "group", vec![Json::from(n), Json::from(total), Json::from(id)]);
        }
        Json::Obj(d)
    }
}

pub fn split_tag_block(line: &str) -> Result<(Option<TagBlock>, &str), UnterminatedTagBlock> {
    let line = py_strip(line);
    let Some(after_open) = line.strip_prefix('\\') else {
        return Ok((None, line));
    };
    let end = after_open.find('\\').ok_or(UnterminatedTagBlock)?;
    let inner = &after_open[..end];
    let remainder = &after_open[end + 1..];

    let (body, hexpart) = match inner.split_once('*') {
        Some((b, h)) => (b, Some(h)),
        None => (inner, None),
    };
    let mut ok = false;
    if let Some(h) = hexpart {
        if h.chars().count() >= 2 {
            let first_two: String = h.chars().take(2).collect();
            ok = py_int_radix(&first_two, 16) == Some(i64::from(checksum(body)));
        }
    }
    let mut params = BTreeMap::new();
    for item in body.split(',') {
        if let Some((code, value)) = item.split_once(':') {
            params.insert(code.to_string(), value.to_string());
        }
    }
    Ok((Some(TagBlock { params, checksum_ok: ok, raw: inner.to_string() }), remainder))
}
```

> **Language notes (Rust)**
> * `str::split_once(sep)` is Rust's `partition`: it returns
>   `Option<(&str, &str)>` — `None` when the separator is absent — so the
>   "was it there?" question becomes an `if let Some((code, value)) = …`
>   pattern match. Both halves are *borrowed slices* of the input; nothing
>   is copied until `to_string()`.
> * `Option` combinators replace the `if x is None: return None` ladders.
>   `self.params.get("c")?` returns `None` from the whole function when the
>   key is missing (the `?` operator works on `Option` inside a function
>   that returns `Option`). `.ok()` converts a `Result` into an `Option`,
>   dropping the error; `.map(String::as_str)` converts
>   `Option<&String>` to `Option<&str>` without touching a `None`.
> * `let Some(x) = expr else { return … };` (a *let-else*, Rust 1.65) binds
>   on success and diverges otherwise — a tidy way to peel off the opening
>   backslash with `strip_prefix`.
> * `BTreeMap<String, String>` is the sorted dictionary of chapter 1;
>   iteration with `for (k, v) in &self.params` yields references, in key
>   order.
> * The function's return type `Result<(Option<TagBlock>, &str), _>` ties
>   the lifetime of the returned `&str` to the argument automatically; the
>   remainder is valid exactly as long as the input line is.

### TypeScript

```typescript
export class TagBlock {
  constructor(
    readonly params: { readonly [code: string]: string },
    readonly checksumOk: boolean = false,
    readonly raw: string = "",
  ) {}

  get time(): number | null {
    const c = this.params["c"];
    return c !== undefined && c !== "" && isDigits(c) ? Number(c) : null;
  }

  get source(): string | null {
    return this.params["s"] ?? null;
  }

  get group(): [number, number, number] | null {
    const g = this.params["g"];
    if (g === undefined || g === "") {
      return null;
    }
    const parts = g.split("-");
    if (parts.length !== 3 || !parts.every(isDigits)) {
      return null;
    }
    return [Number(parts[0]), Number(parts[1]), Number(parts[2])];
  }

  toDict(): JsonObject {
    const params: JsonObject = Object.create(null);
    for (const code of Object.keys(this.params)) {
      params[code] = this.params[code];
    }
    const d: JsonObject = { params, checksum_ok: this.checksumOk };
    const time = this.time;
    if (time !== null) {
      d["time"] = time;
    }
    const group = this.group;
    if (group !== null) {
      d["group"] = [...group];
    }
    return d;
  }
}

export function splitTagBlock(rawLine: string): [TagBlock | null, string] {
  const line = pyStrip(rawLine);
  if (!line.startsWith("\\")) {
    return [null, line];
  }
  const end = line.indexOf("\\", 1);
  if (end < 0) {
    throw new ValueError("unterminated TAG block");
  }
  const inner = line.slice(1, end);
  const [body, star, hexpart] = partition(inner, "*");
  let ok = false;
  if (star && hexpart.length >= 2) {
    try {
      ok = pyHexInt(hexpart.slice(0, 2)) === checksum(body);
    } catch (e) {
      if (!(e instanceof ValueError)) {
        throw e;
      }
      ok = false;
    }
  }
  const params: { [code: string]: string } = Object.create(null);
  for (const item of body.split(",")) {
    const [code, colon, value] = partition(item, ":");
    if (colon) {
      params[code] = value;
    }
  }
  return [new TagBlock(params, ok, inner), line.slice(end + 1)];
}
```

> **Language notes (TypeScript)**
> * `get time(): number | null { … }` declares a *getter*: read as
>   `tag.time` without parentheses, recomputed on each access — the direct
>   counterpart of Python's `@property`.
> * A plain object with a string index signature,
>   `{ [code: string]: string }`, plays the role of `dict`. But a normal
>   `{}` inherits `toString`, `constructor` and `__proto__` from
>   `Object.prototype`; a TAG block with a parameter named `__proto__`
>   (malicious or merely odd) would then *change the object's prototype*
>   instead of storing a value. `Object.create(null)` makes a prototype-less
>   object, which is the safe container for input-derived keys. Reading a
>   missing key yields `undefined`, which the `?? null` operator turns into
>   `null` for a consistent "absent" value.
> * `[number, number, number] | null` is a *tuple type*: a fixed-length
>   array whose element types are known. `[...group]` copies it into a
>   plain array for the JSON writer.
> * `readonly` constructor parameters and the `partition` helper in
>   `common.ts` are as in chapter 1.

**Where the four differ.** The logic is identical; the differences are
all in how "maybe absent" is spelled: Python `X | None` + `None`, C++
`std::optional` + `std::nullopt`, Rust `Option` with `?`/`.ok()`/`.map()`,
TypeScript `T | null` with `??`. Only Rust returns the remainder as a
*view* into the input; the other three copy it. Only Rust reports the
unterminated-block case as a `Result` value; the other three throw.

## 4.5 Sample data and results

`data/ch04-tag-blocks.nmea` (11 lines, derived as described in §4.2.4;
each line's ultimate source is listed in chapter 5 §5.2.2):

```text
\s:003669946,c:1223901985,t:d-74 T25.50,n:1*3A\!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02
\s:003669946,c:1223901985,t:d-103 T25.55,n:2*0D\!AIVDM,1,1,,B,15N2RMPP1cJuFalHdv1`:wvf0@=k,0*14
\s:003669946,c:1223901985,t:d-110 T25.66,n:3*0E\!AIVDM,1,1,,A,14V1HP01BCK5EgBGJCcu=:Rl0@?2,0*5E
\s:003669947,c:1223901985,n:4*4E\!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02
\s:003669947,c:1223901985,n:5*4F\!AIVDM,1,1,,A,14V1HP01BCK5EgBGJCcu=:Rl0@?2,0*5E
\s:003669945,c:1223901981,t:d-103 T25.50,n:6*0B\!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02
\s:003669945,c:1199163535,t:d-107 T54.26,n:7*04\!AIVDM,1,1,1,A,55Mq@Q02=KjmL@;?OODlE8D@U@R18DTp5DE:220lDHN<<7?UNDPSlm3kc3@H88888888880,2*5C
\s:993689982,c:1196666940,g:1-2-1,n:8*0C\!AIVDM,2,1,6,A,CWjqao;wMoB<<grwIguOUrBls;5s7quwNOKgoCLsPSR,0*64
\s:993689982,c:1196666940,g:2-2-1,n:9*0E\!AIVDM,2,2,6,A,LumqliqdL,0*69
\c:1179170462,n:10*28\!AIVDM,1,1,,A,15N91b?P00JRlJ4E56Qe0?v20@14,0*2F
\s:nhcml,c:1305936051,n:11*2F\!AIVDM,1,1,,B,H5MiOp1:uJ0MDhJ0<P4hhDpLE80,2*33
```

Run any of the four decoders:

```sh
python3 code/python/decode.py data/ch04-tag-blocks.nmea
code/cpp/build/decode data/ch04-tag-blocks.nmea
code/rust/target/release/decode data/ch04-tag-blocks.nmea
node code/typescript/dist/decode.js data/ch04-tag-blocks.nmea
```

All four print the ten objects in `tests/expected/ch04-tag-blocks.jsonl`
(ten, not eleven: line 8 is the first fragment of a two-fragment message
and prints nothing on its own). Line 1 in full:

```json
{"ais":{"cog_available":true,"cog_deg":18.0,"comm_state":{"received_stations":5,"scheme":"SOTDMA","slot_timeout":5,"sync_state":0,"sync_state_text":"UTC direct"},"heading_available":true,"lat":42.666992,"lon":-70.459822,"mmsi":538002219,"msg_type":1,"nav_status":0,"nav_status_text":"under way using engine","position_accuracy":1,"position_available":true,"raim":false,"repeat_indicator":0,"rot_deg_per_min":0.0,"rot_raw":0,"rot_text":"not turning","sog_available":true,"sog_knots":14.6,"spare":0,"special_manoeuvre":0,"special_manoeuvre_text":"not available","timestamp":24,"true_heading":18},"channel":"A","fragments":1,"line":1,"own_ship":false,"payload_bits":168,"tagblock":{"checksum_ok":true,"params":{"c":"1223901985","n":"1","s":"003669946","t":"d-74 T25.50"},"time":1223901985},"talker":"AI"}
```

The `"tagblock"` sub-object sits beside `"ais"`, not inside it — the
block describes the *sentence*, not the message. `params` is the raw
dictionary (all strings, keys sorted); `time` is the typed view of `c:`.
Note that the AIS message's own `timestamp` (UTC second of the position
fix, 24) and the receiver's arrival time preserved in `t:` (`T25.50`)
agree to within two seconds, while the logger's `c:` is 12:46:25 UTC — the
`:25` matching `T25.50` is a coincidence of this particular line, as line
6 shows.

Selected fields from the other lines:

| line | `tagblock.params.s` | `tagblock.time` | `tagblock.group` | `tagblock.params.t` | what follows |
|---|---|---|---|---|---|
| 4 | `003669947` | 1223901985 | — | — | the same type 1 as line 1, heard by a second receiver |
| 6 | `003669945` | 1223901981 | — | `d-103 T25.50` | the same type 1 a third time; `c:` is **4 s earlier** although `T` is identical — see chapter 5 §5.2.3 |
| 7 | `003669945` | 1199163535 | — | `d-107 T54.26` | `checksum_ok` is true for the block but the *sentence* reports `"error":"checksum mismatch"` — see §4.6 |
| 9 | `993689982` | 1196666940 | `[2,2,1]` | — | completes the two-fragment type 19 opened by line 8 (`g:1-2-1`); `"fragments":2`, `"payload_bits":312` |
| 10 | *(absent)* | 1179170462 | — | — | the 2007 log carried only a time, so the block has only `c:` and `n:` |
| 11 | `nhcml` | 1305936051 | — | — | a non-numeric source id (a lighthouse receiver's name) |

Line 9's object shows the grouping fields exactly as the decoder emits
them:

```json
"tagblock":{"checksum_ok":true,"group":[2,2,1],"params":{"c":"1196666940","g":"2-2-1","n":"9","s":"993689982"},"time":1196666940}
```

Because the assembler in `vdm.*` still keys on the sentence's own
`(talker, channel, sequential id)` — here `(AI, A, 6)` — the TAG block of
the *completing* fragment (line 9) is what appears in the output; line 8's
block (`g:1-2-1`, `n:8`) is not reported. Exercise 4.7.4 asks you to change
that.

## 4.6 Pitfalls and real-world deviations

* **Backslashes in shells and source code.** `\` is an escape character in
  Python, C++, Rust, TypeScript *and* every common shell. A TAG block typed
  on a command line must be single-quoted (`'\c:…\!AIVDM…'`) and in source
  written `"\\"`. The JSON output doubles it too: a raw line echoed in an
  `"error"` object prints as `"\\c:…"`. None of this changes the data; it
  is all notation.
* **Blocks without a checksum.** Some early implementations emitted
  `\c:1234567890\` with no `*hh`. The standard requires the checksum
  [NMEA 0183 v4.10 §7]; our code still parses the parameters and simply
  reports `checksum_ok:false`, so a downstream consumer can decide how
  strict to be.
* **`c:` in milliseconds vs seconds.** Nothing in the block says which.
  Apply the 10<sup>10</sup> rule (§4.2.2) or, better, know your talker.
  Mixing the two in one database shifts timestamps by a factor of 1000 —
  a 2008 message lands in January 1970.
* **Blocks on their own line.** The standard permits a block with no
  sentence after it (a heartbeat carrying only `c:` and `n:`, or an
  addressing block) [NMEA 0183 v4.10 §7]. `split_tag_block` returns an
  empty remainder; `parse_sentence` then raises `no '$' or '!' in line`,
  and the output object carries both the `"error"` and the `"tagblock"`.
  Do not count such lines as corruption.
* **Unterminated blocks.** A line that begins with `\` and never closes is
  reported as `unterminated TAG block` with the raw text, and nothing else
  is attempted — the sentence boundary is unknown, so guessing would be
  worse than refusing.
* **Mixing TAG blocks with legacy suffixes.** A converter that adds a TAG
  block but forgets to remove the USCG fields after the checksum produces
  `\s:…\!AIVDM,…*02,s34486,d-074,…`. Our decoder handles it (both
  `"tagblock"` and `"uscg"` appear), but the two timestamps may disagree
  and the line is longer than either standard allows. The converter in
  §4.2.4 trims everything after `*hh` for this reason.
* **The 15-character limits.** `s:`, `d:` and `t:` are capped at 15
  characters [NMEA 0183 v4.10 §7]. Some networks put a full host name or a
  URL in `s:`; a strict receiver may reject the block. Our parser does not
  enforce the limit (it reports what it sees) — exercise 4.7.3.
* **The sentence's checksum is still the sentence's problem.** Line 7
  illustrates it: the block verifies (`checksum_ok:true`) but the
  single-sentence type 5 it precedes was "stitched" by a collection tool
  from two fragments, and the tool kept a checksum that does not match the
  joined text; the decoder reports `"error":"checksum mismatch"` with the
  TAG block attached. Chapter 5 §5.6 has the full story.
* **Grouping without `s:`.** A `g:` id is only unique *per talker*. Two
  receivers forwarded through the same server with no `s:` can collide on
  group ids just as they collide on sequential ids. Key on both.

## 4.7 Exercises

1. **Milliseconds (Python).** Extend `uscg_to_tagblock.py` with a
   `--milliseconds` flag that emits `c:` as milliseconds, using the
   fractional part of the logger time when the input has one (lines 10 and
   11 of `data/ch05-uscg-metadata.nmea` do). Then make `TagBlock.time`
   return seconds as a `float` when the raw value exceeds 10<sup>10</sup>,
   and check the two lines round-trip to `1179170462.87` and
   `1305936051.69`.
2. **Hand checksum.** Verify the block checksum of line 11
   (`s:nhcml,c:1305936051,n:11` → `2F`) with pencil and paper, the way
   §4.3 did for line 4. Can you change a *single* character of the body
   without changing the checksum? Which kind of two-character error goes
   undetected? (Hint: XOR is commutative; think about swapping two
   characters.)
3. **Validation (TypeScript).** Add a method `problems(): string[]` to
   `TagBlock` that returns a list of standard violations: block longer than
   80 characters, `s`/`d`/`t` longer than 15, `c` not an integer, `g` not
   three integers, unknown parameter codes. Run it over
   `data/ch04-tag-blocks.nmea`; it should return an empty list for every
   line.
4. **Group-aware assembly (Rust).** Change the `Assembler` in `vdm.rs` so
   that, when the fragment arrived with a TAG block carrying `g:`, the
   assembly key is `(source, group id)` instead of
   `(talker, channel, sequential id)`, and the reported `"tagblock"` is the
   one from the *first* fragment. Lines 8–9 should then report `n:8` and
   `group:[1,2,1]`. Make sure lines without `g:` still assemble as before
   and that `tests/run_tests.sh` tells you precisely what changed.
5. **Reverse converter (C++).** Write `tagblock_to_uscg.cpp` that reads
   `data/ch04-tag-blocks.nmea` and emits chapter 5's suffix format:
   `s:` → `r…`, `c:` → trailing time, `t:` → whatever `d…`/`T…` items it
   contains. Diff the result against `data/ch05-uscg-metadata.nmea` and
   list exactly which fields cannot be recovered (RSSI, receiver counter,
   the six lost decimals of `T`, the fractional logger seconds). This is
   the clearest way to see what the 15-character `t:` limit costs.
6. **Interleaving (any language).** Build an input in which the two
   fragments of line 8–9 are separated by a second copy of the same two
   fragments with `s:003669946` and `g:1-2-7`/`g:2-2-7`. Show that the
   chapter 2 assembler pairs the wrong halves (or, since the payloads are
   identical, *cannot tell*) and that your exercise 4 assembler gets it
   right.

## 4.8 References

* [NMEA 0183] v4.00 (2008) introduced TAG blocks; v4.10 (2012) §7 "TAG
  blocks" is the version described here: delimiters, checksum, 80-character
  limit, parameter codes `c d g n r s t`, 15-character string limits,
  stand-alone blocks, configuration/query sentences.
* [IEC 61162-1] §7.1.2 "TAG block" (ed. 4 onwards) — the same text in the
  international standard; §7.2 sentence structure and 82-character limit;
  §7.2.3 checksum algorithm; §8 VDM/VDO sequential message identifier.
* [AIVDM "NMEA Tag Blocks"] — free description, the
  `g:1-2-73874,n:157036,s:r003669945,c:1241544035` example, notes on the
  unresolved `c:` unit and the unobserved `r:`.
* [AISUser §3.6.2.8] — `Monitor.Connection.n.OutputFormat`: `0` unchanged,
  `1` NMEA 0183 v4.00 TAG block, `2` proprietary USCG R&D Center format;
  §3.6.1.6 `NMEA0183.TAG.Block.Version`.
* Chapter 1 (`nmea.*`: `checksum`, `parse_sentence`), chapter 2 (`vdm.*`:
  `Assembler`), chapter 5 (`uscg.*`: the format the sample file was
  converted from).
