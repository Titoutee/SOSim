import os
import sys


def clear_stdin():
    try:
        if os.name == "nt":
            import msvcrt  # pylint: disable=import-outside-toplevel

            # For Windows systems, Check if there is any pending input in the
            # buffer Discard characters one at a time until the buffer is empty.
            while msvcrt.kbhit():
                msvcrt.getch()
        elif os.name == "posix":
            import select  # pylint: disable=import-outside-toplevel

            # For Unix-like systems, check if there's any pending input in
            # stdin without blocking.
            stdin, _, _ = select.select([sys.stdin], [], [], 0)
            if stdin:
                if sys.stdin.isatty():
                    # pylint: disable=import-outside-toplevel
                    from termios import TCIFLUSH, tcflush

                    # Flush the input buffer
                    tcflush(sys.stdin.fileno(), TCIFLUSH)
                else:
                    # Read and discard input (in chunks).
                    while sys.stdin.read(1024):
                        pass
    except ImportError:
        pass
