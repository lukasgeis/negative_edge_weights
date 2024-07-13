cd $(dirname "$0")
cd ..

python3 -m venv pyenv
. ./pyenv/bin/activate
pip3 install matplotlib==3.8.0 pandas==2.1.2 seaborn==0.13.0
