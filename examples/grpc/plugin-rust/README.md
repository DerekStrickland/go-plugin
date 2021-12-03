# Overview

This Plugin uses GRPC to communicate between a plugin host process and a plugin
written in Rust. However, there are some discrepancies between the way `go-plugin`
works, and the Rust runtime.

## Startup timing

NOTE: This may not be correct

The example Go client can often start and finish before the Rust plugin has a chance
to set up its GRPC Stdio server. This race usually presents itself immediately when
running through a debugger versus executing natively. 

## Stdout/Stderr

During client initialization, the handshake is effectively performed using a channel
in `client.go` (`linesCh`). On the Rust Plugin side, a call to `println!` completes
the handshake. This works effectively since the child process stdout/stderr
are piped back to the client via a `Command` instance.

Unfortunately, these handles are discarded at the end of `client.Start`. So far,
that has been the only reliable way I've been able to receive standard streams 
from the plugin process.

## GRPC Stdio

The GRPC Stdio Server starts, and runs, but the Gag sync doesn't pick up any of
the `stdout` writes from the other processes. So far, I've been unable to devise
a method of monitoring either the `stdout` or `stderr` streams. This results in
the host process being unable to receive the plugin logs. 

## Graceful shutdown

Because the Stdio Server isn't working correctly, the client can't shut down
gracefully.


## Research Links for Stdout/Stderr

https://github.com/krustlet/krustlet/blob/main/crates/wasi-provider/src/wasi_runtime.rs
https://github.com/krustlet/krustlet/blob/main/crates/kubelet/src/log/mod.rs


