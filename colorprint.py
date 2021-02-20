"""
Color print helper
Akseli Lukkarila
2019
"""
import colorama
colorama.init()


class Color:
    red = colorama.Fore.RED
    green = colorama.Fore.GREEN
    yellow = colorama.Fore.YELLOW
    blue = colorama.Fore.BLUE
    magenta = colorama.Fore.MAGENTA
    cyan = colorama.Fore.CYAN
    white = colorama.Fore.WHITE


def get_color(text, color=colorama.Fore.WHITE, bold=False):
    return f"{colorama.Style.BRIGHT if bold else ''}{color}{text}{colorama.Style.RESET_ALL}"


def print_bold(text, color=colorama.Fore.WHITE):
    print(get_color(text, color, True))


def print_color(text, color=colorama.Fore.WHITE, bold=False):
    print(get_color(text, color, bold))


def print_error(text):
    print(f"{get_color('Error', Color.red)}: {text}")
