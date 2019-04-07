# curl-inject-opt

`curl-inject-opt` is a program to inject CURL options into the sessions of a subcommand.
The subcommand will be run with `/path/to/libcurl-inject-opt-preload.so` added to `LD_PRELOAD`,
and a list of options to set in the `CURL_INJECT_OPT` environment variable.
The command-line tool takes care of these steps automatically.

The preloaded library will intercept calls to `curl_easy_perform()`.
Whenever a call is intercepted, the options listed in `CURL_INJECT_OPT` are set on the relevant CURL handle
before the original `curl_easy_perform()` is called.

This can be used to take advantage of certain CURL features even if the program being run doesn't expose them.
Currently, supported options include timeout options, TLS client certificate settings, proxy settings, and `CURLOPT_VERBOSE`.
For a full list, see the table below.

For the exact effects of an option, refer to the man-page of the relevant CURL option.

## Options:

Usage                       |  CURL option                 | Description
----------------------------|------------------------------|---------------
`--verbose <VAL>`           |  `CURLOPT_VERBOSE`           | Set to 1 to enable verbose output from CURL.
`--timeout <VAL>`           |  `CURLOPT_TIMEOUT_MS`        | Timeout in milliseconds for the whole request.
`--connect-timeout <VAL>`   |  `CURLOPT_CONNECTTIMEOUT_MS` | Timeout in milliseconds for the connection phase of the request.
`--proxy <VAL>`             |  `CURLOPT_PROXY`             | Set the proxy to use.
`--proxy-port <VAL>`        |  `CURLOPT_PROXYPORT`         | Set the proxy port.
`--proxy-type <VAL>`        |  `CURLOPT_PROXYTYPE`         | Set the proxy type.
`--proxy-tunnel <VAL>`      |  `CURLOPT_HTTPPROXYTUNNEL`   | Set to 1 to use CONNECT to tunnel through a configured HTTP proxy.
`--no-proxy <VAL>`          |  `CURLOPT_NOPROXY`           | Set hosts to contact directly, bypassing the proxy settings.
`--client-cert <VAL>`       |  `CURLOPT_SSLCERT`           | Use a client certificate to authenticate with a remote server.
`--client-cert-type <VAL>`  |  `CURLOPT_SSLCERTTYPE`       | Specify the type of the client certificate (normally defaults to PEM).
`--client-key <VAL>`        |  `CURLOPT_SSLKEY`            | Use a separate file as key with the client certificate.


## Building

To build the project, all you need is `make`, `bash` and `cargo`, the Rust build tool.
Cargo will take care of downloading additional Rust dependencies.
See the `Cargo.toml` files to get a full list of Rust dependencies.

Since the command-line application needs to know where the preload library will be installed,
there is also a `./configure` script included.

To build and install the project, run the following commands:

```console
$ ./configure PREFIX=/usr
$ make
$ make install
```

Run `./configure --help` for an overview of all supported compile-time configuration options.
The build system also supports installing to a staging directory for packaging purposes:

```console
$ make install DESTDIR="..."
```

## Security considerations

Since `curl-inject-opt` uses `LD_PRELOAD` to intercept function calls,
it is subject to the usual security restrictions imposed by the operating system.
On Linux, this means that programs which are run in secure-execution mode will not simply preload the library.
This is generally not a problem, since it should work fine to run `sudo curl-inject-opt ...`.

Secure execution mode on Linux is used (amongst others) when a program has the `setuid` or `setgid` permission bit set,
or if the program has additional capabilities as set by the `setcap` tool.
For more information, see `man 8 ld.so` on Linux.

Although not recommended, it is possible to make `curl-inject-opt` work even with commands that are subject to the restrictions of secure-execution mode.

To make `curl-inject-opt` work with secure execution mode, the entry added to `LD_PRELOAD` must consist only of the library name `libcurl-inject-opt.so` with no path information.
The library must be found by the dynamic linker in the default search path and the library must have the `setuid` permission bit set.
This can party be achieved by configuring the project with `./configure PREFIX=/usr RELY_ON_SEARCH=1`.
See `./configure --help` for more information.

It is up to the packager or installer to set the `setuid` bit of the installed library, if desired.
This is not done by the build system.
