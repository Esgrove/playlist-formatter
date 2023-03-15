import sys

from PlaylistFormatter import PlaylistFormatter
from PlaylistGui import RunGui

if __name__ == "__main__":
    if len(sys.argv) > 1:
        # arguments given, run on command line
        args = sys.argv[1:]
        filename = args[0]
        outfile = args[1] if len(args) >= 2 else None

        formatter = PlaylistFormatter()
        formatter.read_playlist(filename)
        formatter.print_playlist()
        formatter.export_csv(outfile)
    else:
        RunGui()
