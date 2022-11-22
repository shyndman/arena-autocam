#!/bin/bash

set -x
set -e

apt-get install software-properties-common;
curl -O 'https://archive.raspberrypi.org/debian/pool/main/r/raspberrypi-archive-keyring/raspberrypi-archive-keyring_2016.10.31_all.deb';
dpkg -i ./raspberrypi-archive-keyring_2016.10.31_all.deb;
apt-add-repository "deb http://archive.raspberrypi.org/debian/ bullseye main";
apt-get update &&
apt-get install 'libpigpio1=1.79-1+rpt1';
