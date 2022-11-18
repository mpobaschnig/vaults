#!/bin/sh

exec flatpak-spawn --host umount "$@"
