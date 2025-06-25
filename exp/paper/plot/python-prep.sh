#!/bin/bash

# Change into repo-folder
cd $(dirname "$0")
cd ../..

# Create python environment for plotting
python3 -m venv pyplotenv
. ./pyplotenv/bin/activate
pip3 install matplotlib==3.8.0 pandas==2.1.2 seaborn==0.13.0
