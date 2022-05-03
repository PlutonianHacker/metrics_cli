#!/bin/bash 

echo "Installing metrics cli"

cargo build --release

path="$PWD/target/release/metrics"

chmod +x $path

cp $path ~/.local/bin
