"""The one line discipline every cuesheet format shares."""


def meaningful_lines(text):
    """The stripped, non-blank lines of a recorded text, in order."""
    return [line.strip() for line in text.split("\n") if line.strip()]
