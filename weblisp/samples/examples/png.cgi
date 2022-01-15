#!/bin/sh

PNG=${HOME}/picture-language/sicp/sicp.png 

if [ -f $PNG ]
then
  echo "Content-Type: image/png"
  echo "Status: 200"
  echo 
  cat $PNG
else
  echo "Content-Type: text/plain"
  echo "Status: 404"
  echo
  echo "$PNG is not found."
fi 
