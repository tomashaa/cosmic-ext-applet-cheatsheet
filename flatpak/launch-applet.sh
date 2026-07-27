#!/bin/sh
# Launch helper for COSMIC panel when running as Flatpak.
set -eu
export PATH="${PATH}:/app/bin"
exec cosmic-ext-applet-cheatsheet "$@"
