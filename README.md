linux-cec
=========

linux-cec is a collection of Rust crates intended for interfacing with Linux's
[HDMI CEC](https://en.wikipedia.org/Consumer_Electronics_Control) (Consumer Electronics Control) userspace subsystem:

- linux-cec: A library for interacting with CEC devices via HDMI adapters connected to the host.
- linux-cec-sys: Interfaces relating to the underlying ioctl API for /dev/cec* nodes, translated from
  `include/linux/cec.h` into Rust. You will probably not need to use these, but they are separated out for developers
  who don't wish to use linux-cec directly.
- linux-cec-macros: A private library used for implementing various aspects of the linux-cec crate.
  You should not use this directly, and no stability is guaranteed between versions.
- cecd: A daemon that runs on the host and exposes high level control for CEC over DBus, optionally exposing an input
  device for the remote control via [uinput](https://www.kernel.org/doc/html/latest/input/uinput.html).

Note that most GPUs do not expose CEC to the OS, and as such you may need an adapter supported by the kernel to use
CEC. Please see the Linux admin guide [page on HDMI CEC](https://docs.kernel.org/admin-guide/media/cec.html) for more
details on supported devices. This repo also contains versions of the systemd units and udev rules to enable
auto-binding of external adapters. These are located `data` directory and can be installed into
`/usr/lib/systemd/system/` and `/usr/lib/udev/rules.d/` respectively.

## Why not libCEC FFI?

[LibCEC](http://libcec.pulse-eight.com/) has been the de facto library for interacting with CEC devices across
multiple operating systems for years. However, there are some significant shortcomings with the library. The most
important one is that the library has been unmaintained for years, with no releases since 2020, and is rife with
issues that could use tackling. However, forking the library is hampered by the fact that libCEC is itself claimed to
be a registered trademark of Pulse-Eight per the source. The library itself is also GPL2+, which restricts the range
of software that can directly use it.

Some of these issues apply specifically to the Linux CEC API. For example, it's hardcoded to only support /dev/cec0,
and does not support hotplugging. If the device is disconnected while in use by an application, a thread will just
continuously poll the device, get an error, and then try again. This cannot be programmatically detected except for
scanning log messages. This is anything but robust, and there wasn't really a good way to fix this without some
massive overhauls.

Given all of these factors, it seemed like a fresh start specifically targeting the Linux subsystem
instead of a more pluggable system. Many of the devices libCEC supports have since gotten kernel drivers as well, so
the need for a userspace library handling it all directly has diminished since libCEC was started.

## cecd

Many features of HDMI CEC make sense at a system-wide level instead of an application level. For example, handling
wake/suspend synchronization of the device and the TV. Furthermore, some application-level features can make sense to
expose without directly using the library, such as mapping remote control button presses to key inputs. Cecd aims to
support all of these features:

- Optional support for waking the TV when coming out of suspend and putting the TV to sleep when entering suspend.
- Translating a configurable set of UI buttons (usually remote controller buttons) to an evdev input device.
- A DBus service for sending and receiving messages, including a handful of convenience methods.

---

linux-cec is copyright © 2024, Valve Software.
Excluding the linux-cec-sys crate, all crates are licensed under the LGPL 2.1 or newer. linux-cec-sys is licensed
under the BSD 3-clause license and incorporates code written by Cisco Systems, Inc. for the Linux kernel.
