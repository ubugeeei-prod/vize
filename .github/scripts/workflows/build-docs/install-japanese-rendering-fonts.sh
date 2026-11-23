#!/usr/bin/env bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends fonts-noto-cjk
fc-match --format '%{family}\n' ':lang=ja'
