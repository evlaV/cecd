linux-cec
=========

linux-cec is a collection of Rust crates intended for interfacing with Linux's
[HDMI CEC](https://en.wikipedia.org/Consumer_Electronics_Control) (Consumer Electronics Control) userspace subsystem:

- linux-cec: A library for interacting with CEC devices via HDMI adapters connected to the host.
- linux-cec-sys: Interfaces relating to the underlying ioctl API for /dev/cec* nodes, translated from
  `include/linux/cec.h` into Rust. You will probably not need to use these, but they are separated out for developers
  who don't wish to use linux-cec directorly.
- linux-cec-macros: A private library used for implementing various aspects of the linux-cec crate.
  You should not use this directly, and no stability is guaranteed between versions.
- cecd: A daemon that runs on the host and exposes high level control for CEC over DBus, optionally exposing an input
  device for the remote control via [uinput](https://www.kernel.org/doc/html/latest/input/uinput.html).

Note that most GPUs do not expose CEC to the OS, and as such you may need an adapter supported by the kernel to use
CEC. Please see the Linux admin guide [page on HDMI CEC](https://docs.kernel.org/admin-guide/media/cec.html) for more
details on supported devices. This repo also contains versions of the systemd units and udev rules to enable
auto-binding of external adapters. These are located `data` directory and can be installed into
`/usr/lib/systemd/system/` and `/usr/lib/udev/rules.d/` respectively.

linux-cec is copyright © 2024, Valve Software.
Excluding the linux-cec-sys crate, all crates are licensed under the LGPL 2.1 or newer. linux-cec-sys is licensed
under the BSD 3-clause license and incorporates code written by Cisco Systems, Inc. for the Linux kernel.
