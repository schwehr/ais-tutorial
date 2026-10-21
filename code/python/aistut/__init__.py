"""aistut — the AIS tutorial's teaching decoder (Python version).

Requires **Python 3.14 or newer**. The code relies on

* PEP 604 union syntax in annotations (`int | None` instead of
  `Optional[int]`),
* PEP 585 built-in generics (`list[str]`, `dict[int, str]`) and
  `collections.abc.Callable`,
* PEP 695 `type` aliases (`type Decoder = Callable[...]`), and
* PEP 649/749 deferred evaluation of annotations, which is why no module
  needs `from __future__ import annotations` even where a method's return
  annotation names its own class.

The command-line wrapper (`code/python/decode.py`) checks the interpreter
version and stops with a clear message on older Pythons:

    if sys.version_info < (3, 14):
        sys.exit("This tutorial's Python code requires Python 3.14 or newer")
"""
