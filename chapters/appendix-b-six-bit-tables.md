# Appendix B — Six-bit tables

Two different six-bit codes appear in AIS, and confusing them is the most common beginner's mistake:

1. **Payload armouring** turns 6 bits of *any* AIS message into one printable character of the `!AIVDM` payload field. Decode: subtract 48 from the ASCII code; if the result is greater than 40, subtract 8 more [IEC 61162-1 §7.3.3], [AIVDM "Payload Armoring"].

2. **Six-bit ASCII** is the character set used *inside* text fields (ship name, call sign, destination, safety text, AtoN name) [M.1371-5 Annex 8 Table 47]. `@` (0) is the padding character; lower-case letters cannot be represented.

## B.1 Payload armouring (sentence payload character ↔ 6-bit value)

| Char | ASCII | Value | Bits | | Char | ASCII | Value | Bits |
|---|---|---|---|---|---|---|---|---|
| `0` | 48 | 0 | `000000` | | `P` | 80 | 32 | `100000` |
| `1` | 49 | 1 | `000001` | | `Q` | 81 | 33 | `100001` |
| `2` | 50 | 2 | `000010` | | `R` | 82 | 34 | `100010` |
| `3` | 51 | 3 | `000011` | | `S` | 83 | 35 | `100011` |
| `4` | 52 | 4 | `000100` | | `T` | 84 | 36 | `100100` |
| `5` | 53 | 5 | `000101` | | `U` | 85 | 37 | `100101` |
| `6` | 54 | 6 | `000110` | | `V` | 86 | 38 | `100110` |
| `7` | 55 | 7 | `000111` | | `W` | 87 | 39 | `100111` |
| `8` | 56 | 8 | `001000` | | ``` | 96 | 40 | `101000` |
| `9` | 57 | 9 | `001001` | | `a` | 97 | 41 | `101001` |
| `:` | 58 | 10 | `001010` | | `b` | 98 | 42 | `101010` |
| `;` | 59 | 11 | `001011` | | `c` | 99 | 43 | `101011` |
| `<` | 60 | 12 | `001100` | | `d` | 100 | 44 | `101100` |
| `=` | 61 | 13 | `001101` | | `e` | 101 | 45 | `101101` |
| `>` | 62 | 14 | `001110` | | `f` | 102 | 46 | `101110` |
| `?` | 63 | 15 | `001111` | | `g` | 103 | 47 | `101111` |
| `@` | 64 | 16 | `010000` | | `h` | 104 | 48 | `110000` |
| `A` | 65 | 17 | `010001` | | `i` | 105 | 49 | `110001` |
| `B` | 66 | 18 | `010010` | | `j` | 106 | 50 | `110010` |
| `C` | 67 | 19 | `010011` | | `k` | 107 | 51 | `110011` |
| `D` | 68 | 20 | `010100` | | `l` | 108 | 52 | `110100` |
| `E` | 69 | 21 | `010101` | | `m` | 109 | 53 | `110101` |
| `F` | 70 | 22 | `010110` | | `n` | 110 | 54 | `110110` |
| `G` | 71 | 23 | `010111` | | `o` | 111 | 55 | `110111` |
| `H` | 72 | 24 | `011000` | | `p` | 112 | 56 | `111000` |
| `I` | 73 | 25 | `011001` | | `q` | 113 | 57 | `111001` |
| `J` | 74 | 26 | `011010` | | `r` | 114 | 58 | `111010` |
| `K` | 75 | 27 | `011011` | | `s` | 115 | 59 | `111011` |
| `L` | 76 | 28 | `011100` | | `t` | 116 | 60 | `111100` |
| `M` | 77 | 29 | `011101` | | `u` | 117 | 61 | `111101` |
| `N` | 78 | 30 | `011110` | | `v` | 118 | 62 | `111110` |
| `O` | 79 | 31 | `011111` | | `w` | 119 | 63 | `111111` |

ASCII 88–95 (`X Y Z [ \ ] ^ _`) are **not** valid payload characters. The gap exists because early equipment could not transmit them reliably; a decoder should reject them.

## B.2 Six-bit ASCII for text fields (6-bit value ↔ character)

| Value | Bits | Char | | Value | Bits | Char | | Value | Bits | Char | | Value | Bits | Char |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | `000000` | `@` (pad) | | 16 | `010000` | `P` | | 32 | `100000` | space | | 48 | `110000` | `0` |
| 1 | `000001` | `A` | | 17 | `010001` | `Q` | | 33 | `100001` | `!` | | 49 | `110001` | `1` |
| 2 | `000010` | `B` | | 18 | `010010` | `R` | | 34 | `100010` | `"` | | 50 | `110010` | `2` |
| 3 | `000011` | `C` | | 19 | `010011` | `S` | | 35 | `100011` | `#` | | 51 | `110011` | `3` |
| 4 | `000100` | `D` | | 20 | `010100` | `T` | | 36 | `100100` | `$` | | 52 | `110100` | `4` |
| 5 | `000101` | `E` | | 21 | `010101` | `U` | | 37 | `100101` | `%` | | 53 | `110101` | `5` |
| 6 | `000110` | `F` | | 22 | `010110` | `V` | | 38 | `100110` | `&` | | 54 | `110110` | `6` |
| 7 | `000111` | `G` | | 23 | `010111` | `W` | | 39 | `100111` | `'` | | 55 | `110111` | `7` |
| 8 | `001000` | `H` | | 24 | `011000` | `X` | | 40 | `101000` | `(` | | 56 | `111000` | `8` |
| 9 | `001001` | `I` | | 25 | `011001` | `Y` | | 41 | `101001` | `)` | | 57 | `111001` | `9` |
| 10 | `001010` | `J` | | 26 | `011010` | `Z` | | 42 | `101010` | `*` | | 58 | `111010` | `:` |
| 11 | `001011` | `K` | | 27 | `011011` | `[` | | 43 | `101011` | `+` | | 59 | `111011` | `;` |
| 12 | `001100` | `L` | | 28 | `011100` | `\` | | 44 | `101100` | `,` | | 60 | `111100` | `<` |
| 13 | `001101` | `M` | | 29 | `011101` | `]` | | 45 | `101101` | `-` | | 61 | `111101` | `=` |
| 14 | `001110` | `N` | | 30 | `011110` | `^` | | 46 | `101110` | `.` | | 62 | `111110` | `>` |
| 15 | `001111` | `O` | | 31 | `011111` | `_` | | 47 | `101111` | `/` | | 63 | `111111` | `?` |

Notes: values 0–31 are ASCII 64–95 (`@` to `_`), values 32–63 are ASCII 32–63 (space to `?`). To convert a value to its ASCII code: `code = value + 64 if value < 32 else value`. Trailing `@` characters are padding and are stripped; many encoders pad with spaces instead, so strip trailing spaces too [AIVDM "AIS Payload Data Types"].

## B.3 Worked example

Payload `15N9` armours to values 1, 5, 30, 9 → bits `000001 000101 011110 001001`. The first six bits are the message type (1); the next two bits (`00`) the repeat indicator; the MMSI starts at bit 8. The six-bit *text* string `TEST@@` would be values 20, 5, 19, 20, 0, 0 — a completely different mapping.

