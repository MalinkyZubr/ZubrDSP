#!/bin/bash
# call me with source

if [ "$1" == "-t" ]; then
  export RUST_LOG=ZubrDSP=trace
  echo "Rust logging set to trace for ZubrDSP"
elif [ "$1" == "-f" ]; then
  export RUST_LOG=ZubrDSP=error
  echo "Rust logging reset to error for ZubrDSP"
else
  echo "use the flag -t to set logging to trace, and -f to set it to error for ZubrDSP"
fi