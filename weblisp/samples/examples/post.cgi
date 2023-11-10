#!/usr/bin/python
import sys
from os import environ

print("Content-Type: text/plain")
print("Status: 200")
print("")

line = sys.stdin.readline()
print(line.rstrip("\n"))
print(environ["QUERY_STRING"])
