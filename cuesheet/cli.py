"""The command line tool. It lists and never executes."""

import argparse
import sys
from pathlib import Path

from .catalogue import parse_catalogue
from .context import parse_context
from .errors import CuesheetError
from .sheet import build_sheet, format_sheet


def _read(path, what):
    try:
        return Path(path).read_bytes().decode("utf-8")
    except (OSError, UnicodeDecodeError):
        raise CuesheetError(f"the {what} file {path} cannot be read")


def main(argv=None):
    parser = argparse.ArgumentParser(prog="cuesheet")
    sub = parser.add_subparsers(dest="command", required=True)
    sheet_parser = sub.add_parser("sheet")
    sheet_parser.add_argument("catalogue")
    sheet_parser.add_argument("context")
    args = parser.parse_args(argv)
    try:
        catalogue = parse_catalogue(
            _read(args.catalogue, "catalogue"), args.catalogue
        )
        context = parse_context(
            _read(args.context, "context"), args.context, catalogue
        )
    except CuesheetError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    cues = build_sheet(catalogue, context)
    sys.stdout.write(format_sheet(context, cues))
    return 0
