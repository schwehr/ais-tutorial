# AGENTS.md — `sample-data/tide1/OldTideData/`

Eight files `2006_05_24.TXT` … `2007_01_23.TXT` (17 MB, 567 000 lines): the
same JEL tide gauge captured *before* `serial_logger.py` existed, with a
Windows terminal program. Same text block format as `../tide/` (`Date/Time:
6.05.24 15:23:20`, `01 Reference`, `02 Water level … m`, `03 Water
temperature … Deg.C`) but with `\r` sprinkled at the *start* of lines and
trailing spaces. Each file spans several weeks (file name = start date). No
logger timestamps; rely on the gauge's own `Date/Time`.
