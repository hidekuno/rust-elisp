#!/usr/bin/env python
#
# this is toy tool
#
# [notice]
#   To run this program, python must be built with clang.
#   ex.)
#     sudo apt install clang
#     CC=clang pyenv install 3.11.6
#
# hidekuno@gmail.com
#
import ctypes
import os
import sys
from enum import Enum

PROMPT="<rust-elisp> "

sofile = os.path.join(
    os.environ["HOME"], "rust-elisp", "ffilisp", "target", "release", "libffilisp.so"
)
if not os.path.isfile(sofile):
    sys.exit(1)

rust = ctypes.cdll.LoadLibrary(sofile)
rust.do_scheme.argtypes = [ctypes.c_char_p]
rust.do_scheme.restype = ctypes.c_void_p
rust.free_scheme_result.argtypes = [ctypes.c_void_p]


def do_scheme(ex):
    p = ex.encode("utf-8")
    ptr = rust.do_scheme(p)
    if ptr is None:
        return None
    try:
        res_bytes = ctypes.string_at(ptr)
        return res_bytes.decode("utf-8")
    finally:
        rust.free_scheme_result(ptr)

def count_parenthesis(program):
    class CharMode(Enum):
        INIT = 0
        WHITESPACE = 1
        SHARP = 2
        BACKSLASH = 3

    left, right = 0,0
    pre = ''
    str_mode = False
    char_mode = CharMode.WHITESPACE

    for c in program:
        if str_mode:
            if pre != '\\' and  c == '"':
                str_mode = False
        elif char_mode == CharMode.BACKSLASH:
            char_mode = CharMode.INIT
        elif pre != '\\' and  c == '"':
            str_mode = True
        elif c.isspace() and char_mode == CharMode.INIT:
            char_mode = CharMode.WHITESPACE
        elif c == '#' and char_mode == CharMode.WHITESPACE:
            char_mode = CharMode.SHARP
        elif c == '\\' and char_mode == CharMode.SHARP:
            char_mode = CharMode.BACKSLASH
        elif c == '(':
            left += 1
        elif c == ')':
            right += 1
        pre = c

    return (left, right)

def repl():
    print("######## This is a mini lisp like Scheme. #######\n")

    program=""
    prompt=PROMPT
    while True:
        try:
            ex = input(prompt)
            if ex == "(quit)":
                break
            if ex == "":
                continue

            program += ex + "\n"
            left, right = count_parenthesis(program)
            if left > right:
                prompt=""
                continue

            value = do_scheme(program)
            print(value)
            prompt=PROMPT
        except KeyboardInterrupt:
            print("")
            continue
        except EOFError:
            break

        program=""
repl()
