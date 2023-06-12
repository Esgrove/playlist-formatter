#!/usr/bin/env python3
# Wrapper for running Python GUI or CLI.
import sys

from PlaylistFormatter.PlaylistCli import run_cli
from PlaylistFormatter.PlaylistGui import run_gui

if __name__ == "__main__":
    if len(sys.argv) > 1:
        # arguments given, run on command line
        run_cli(sys.argv[1:])
    else:
        run_gui()
