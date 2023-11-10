#!/usr/bin/python
from os import environ

print("Content-Type: text/plain")
print("Status: 301")
print("Location: https://www.yahoo.co.jp/")
print("")

for k, v in environ.items():
    print("%s=%s" % (k, v))
