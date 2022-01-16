#!/usr/bin/python
import sys
from os import environ

print("Content-Type: text/plain")
print("Status: 200")
print("")

for (k,v) in environ.items():
	print("%s=%s" % (k, v))
