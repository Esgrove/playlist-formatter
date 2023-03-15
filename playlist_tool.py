#!/usr/bin/env python3
import sys

from PlaylistFormatter.PlaylistCli import run_cli
from PlaylistFormatter.PlaylistGui import run_gui

if __name__ == "__main__":
    # arguments given, run on command line
    if len(sys.argv) > 1:
        run_cli()
    else:
        run_gui()
