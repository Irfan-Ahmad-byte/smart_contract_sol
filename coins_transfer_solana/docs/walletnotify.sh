#!/bin/sh
curl -d "txid=$1" http://127.0.0.1:6100/litecoin/notify-transaction