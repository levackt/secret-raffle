# Secret Raffle

## DEMO RAFFLE
Simple raffle demo 

## Description
This is a simple raffle game. The 'Raffle Host' will deploy an instance of this contract. 

Anyone can join the raffle, by submitting a transaction from their account, the more funds you send the greater your odds of winning. 
You can enter as many times as you like.

When the raffle host decides to end the raffle, a winner will be chosen at random from all the accounts that entered.


## Usage

### As a participant 

#### Join the raffle

To join, you simply submit a `join` transaction, and choose a lucky phrase or number (and keep it secret!). This will be used as entropy for the required randomness.

```bash
secretcli tx compute execute '{ "join": { "phrase": "<write something fun here>" }}' --from account --label raffle
```

`phrase` is expected to be a string, so choose whatever you want as long as you don't forget to surround it by double quotes
For example:
* right: `"5"` 
* wrong: `5`

#### Did I join?
Check if an address was successfully entered in the raffle
```
secretcli q compute query <contract-address> '{"joined": {"address": "<your address>"}}'
```

#### See who won
See who was selected as the winner
```
secretcli q compute query <contract-address> '{"winner": {}}'
```

### As a raffle host

### Store the contract on-chain
```bash
secretcli tx compute store contract.wasm.gz --from account --gas auto
```

#### Instantiate contract
```bash
secretcli tx compute instantiate <code_id> '{"seed": "<some long secret here>"}' --label <label> --from account
```

#### End raffle - will select a winner
```bash
secretcli tx compute execute <contract-address> '{ "end_lottery": {"winner_to_select": <1-3>} }' --from account
```

#### Javascript
The /client directory has the above as JS, you can deploy and run a raffle as follows;

```bash

# start docker 
docker run -it --rm \
 -p 26657:26657 -p 26656:26656 -p 1317:1317 \
 -v /Users/taariq/code/enigma-protocol:/root/code \
 --name secretdev enigmampc/secret-network-sw-dev:v1.0.2

# in a new terminal, start rest-server
docker exec secretdev \
  secretcli rest-server \
  --node tcp://localhost:26657 \
  --trust-node \
  --laddr tcp://0.0.0.0:1317 


cd client

chmod +x ./scripts/fund_accounts.sh

./scripts/fund_accounts.sh

yarn

yarn deploy-contract
```

For more details, check out the [messages module](https://github.com/levackt/secret-raffle/blob/master/src/msg.rs).

### Troubleshooting 

All transactions are encrypted, so if you want to see the error returned by a failed transaction, you need to use the command

`secretcli q compute tx <TX_HASH>`
