#!/bin/bash
docker exec secretdev \
  secretcli tx send a secret1cdycaskx8g9gh9zpa5g8ah04ql0lzkrsxmcnfq 10000000000uscrt -y -b block \
  --keyring-backend test

docker exec secretdev \
  secretcli tx send a secret1wr7w7e84ay7v4jzeyncka95ywky6w7azf202c0 10000000000uscrt -y -b block \
  --keyring-backend test

docker exec secretdev \
  secretcli tx send a secret18fh4tre2l04lc32kqqu69uu5aw6xr03dn69hrk 10000000000uscrt -y -b block \
  --keyring-backend test%
