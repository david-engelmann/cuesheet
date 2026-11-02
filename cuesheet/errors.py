"""Error types for cuesheet."""


class CuesheetError(Exception):
    """Base class for every refusal cuesheet raises."""


class CatalogueError(CuesheetError):
    """A catalogue is malformed."""


class ContextError(CuesheetError):
    """A context is malformed or belongs to another harness family."""
