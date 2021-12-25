#!/bin/sh

if [ "$#" -eq 2 ]; then
    exec flatpak-spawn --host --env=CRYFS_FRONTEND=noninteractive cryfs "$@"
else
    exec flatpak-spawn --host cryfs "$@"
fi
