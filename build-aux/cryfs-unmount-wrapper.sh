#!/bin/sh

exec flatpak-spawn --host cryfs-unmount "$@"
