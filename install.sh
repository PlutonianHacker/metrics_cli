#!/bin/bash 

echo "Installing metrics cli"

cargo build --release || 0

path="$PWD/target/release/metrics"

chmod +x $path

cp $path ~/.local/bin
