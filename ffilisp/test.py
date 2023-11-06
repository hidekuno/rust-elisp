import unittest
import ctypes

rust = ctypes.cdll.LoadLibrary("target/release/libffilisp.so")

def do_scheme(ex):
    p = ctypes.create_string_buffer(ex.encode('utf-8'))
    r = rust.do_scheme(p)
    return ctypes.c_char_p(r).value.decode('utf-8')

class TestMethods(unittest.TestCase):
    # the testing framework will automatically call for every single test
    def setUp(self):
        pass

    # the testing framework will automatically call for every single test
    def tearDown(self):
        pass

    def test_calc(self):
        self.assertEqual("6",do_scheme("(+ 1 2 3)"))

    def test_define(self):
        self.assertEqual("a",do_scheme("(define a 100)"))
        self.assertEqual("2000",do_scheme("(* a 20)"))

    def test_lambda(self):
        self.assertEqual("test",do_scheme("(define test (lambda (a b)(+ a b)))"))
        self.assertEqual("30",do_scheme("(test 10 20)"))

if __name__ == '__main__':
    try:
        unittest.main()
    except Exception as e:
        print(e, file=sys.stderr)
