#!/bin/bash
cargo build --release

mv result.json result.old.json
target/release/cycle_cover --nodes  8 --steps 40 --runs 10 --repetitions 500 --quit-early 2>> result.json
target/release/cycle_cover --nodes 12 --steps 40 --runs 10 --repetitions 100 --quit-early 2>> result.json
target/release/cycle_cover --nodes 16 --steps 40 --runs 10 --repetitions  50 --quit-early 2>> result.json

./plot.py