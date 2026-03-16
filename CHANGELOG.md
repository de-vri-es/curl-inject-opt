v0.2.4 - 2026-03-16:
  * Replace `StructOp` with `clap`.
  * Use `LazyLock` instead of mutable static.
  * Replace `libc::isatty` with `std::io::IsTerminal`.
  * Remove unnecessary unsafe code for parsing integer options.

v0.2.3 - 2023-01-26:
  * Fix argument parsing with clap 4.

v0.2.2 - 2022-11-10:
  * Update dependencies.

v0.2.1:
  * Add support for CURL multi handles.

v0.2.0:
  * Fix spelling of --client-key-type option and add it to the README.
  * Tweak README.md security considerations.
  * Update README.md for --timeout and --connect-timeout.

v0.1.1:
  * Add --timeout and --connect-timeout options.
  * Add --client-key-type option.
  * Make option descriptions terser.

v0.1.0:
  * First release.
