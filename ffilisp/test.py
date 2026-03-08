import ctypes
import sys
import unittest

rust = ctypes.cdll.LoadLibrary("target/release/libffilisp.so")
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

class TestMethods(unittest.TestCase):
    # the testing framework will automatically call for every single test
    def setUp(self):
        pass

    # the testing framework will automatically call for every single test
    def tearDown(self):
        pass

    def test_calc(self):
        self.assertEqual("6", do_scheme("(+ 1 2 3)"))

    def test_define(self):
        self.assertEqual("a", do_scheme("(define a 100)"))
        self.assertEqual("2000", do_scheme("(* a 20)"))

    def test_lambda(self):
        self.assertEqual("test", do_scheme(
            "(define test (lambda (a b)(+ a b)))"))
        self.assertEqual("30", do_scheme("(test 10 20)"))


if __name__ == "__main__":
    try:
        unittest.main()
    except Exception as e:
        print(e, file=sys.stderr)
