#!/usr/bin/python
import sys
from os import environ

print("Content-type: text/plain")
print("")
if len(sys.argv) > 1:
	print(sys.argv[1])
for (k,v) in environ.items():
	print("%s=%s" % (k, v))

