# Third-party notices

microlith has **12 runtime dependencies**. This file exists for one vendored
file, plus the trademark acknowledgements a public project should make.

## FORMAT.md — vendored from cavekit

[`FORMAT.md`](../FORMAT.md) is not microlith's work. It is the cavekit `SPEC.md`
format specification, version 4.1.0, vendored here **byte-for-byte** so that a
tool citing the format ships the format it cites. It is not forked, not
generated, and not edited — any change to it is a re-sync from upstream, never a
local patch.

- Upstream: <https://github.com/JuliusBrussee/cavekit>
- Copyright (c) 2026 Julius Brussee
- Licensed under the MIT License, reproduced in full below

What microlith *enforces* lives in its own [`SPEC.md`](../SPEC.md) under §V and is
a superset of the vendored document. The format is an input, not the identity.

<!-- BEGIN crates -->
| Crate | Version | License |
| --- | --- | --- |
| anstyle | 1.0.14 | MIT OR Apache-2.0 |
| clap | 4.6.7 | MIT OR Apache-2.0 |
| clap_builder | 4.6.7 | MIT OR Apache-2.0 |
| clap_derive | 4.6.7 | MIT OR Apache-2.0 |
| clap_lex | 1.1.1 | MIT OR Apache-2.0 |
| heck | 0.5.0 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 (*) |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 (*) |
| syn | 3.0.9 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
<!-- END crates -->

### MIT License — cavekit

```text
MIT License

Copyright (c) 2026 Julius Brussee

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Trademarks

Nominative use only; no affiliation or endorsement is implied.

- **Rust** and **Cargo** are trademarks of the Rust Foundation.
- **Linux** is a registered trademark of Linus Torvalds.
- **GNOME** is a trademark of the GNOME Foundation.
- **NixOS** and **Nix** are trademarks of the NixOS Foundation.
- **GitHub** is a trademark of GitHub, Inc.
- **Claude** and **Anthropic** are trademarks of Anthropic PBC.

## microlith itself

Everything else in this repository is licensed under the MIT License — see
[`LICENSE`](../LICENSE).
