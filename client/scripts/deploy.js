#!/usr/bin/env node

const { Encoding } = require("@iov/encoding");
const { coin } = require("@cosmjs/sdk38");

/* eslint-disable @typescript-eslint/camelcase */
const { EnigmaUtils, Secp256k1Pen, SigningCosmWasmClient, pubkeyToAddress, encodeSecp256k1Pubkey } = require("secretjs");
const fs = require("fs");

const httpUrl = "http://localhost:1317";
// const httpUrl = "https://bootstrap.secrettestnet.io";

const account = {
  mnemonic:
    "economy stock theory fatal elder harbor betray wasp final emotion task crumble siren bottom lizard educate guess current outdoor pair theory focus wife stone",
  address: "secret1cdycaskx8g9gh9zpa5g8ah04ql0lzkrsxmcnfq",
};
const account2 = {
  mnemonic:
    "caution grocery mobile news extend tooth coyote main foot outside pipe omit ugly squirrel stay decline furnace verify unfold vote subway adapt spawn drop",
  address: "secret1wr7w7e84ay7v4jzeyncka95ywky6w7azf202c0",
};
const account3 = {
  mnemonic:
    "notice laundry key exit youth harbor enroll current happy clog fury rule deputy business scrap toe alone atom hover sadness boat physical broom resource",
  address: "secret18fh4tre2l04lc32kqqu69uu5aw6xr03dn69hrk",
};
const accounts = [account, account2, account3];

const customFees = {
  upload: {
    amount: [{ amount: "2000000", denom: "uscrt" }],
    gas: "2000000",
  },
  init: {
    amount: [{ amount: "500000", denom: "uscrt" }],
    gas: "500000",
  },
  exec: {
    amount: [{ amount: "500000", denom: "uscrt" }],
    gas: "500000",
  },
  send: {
    amount: [{ amount: "80000", denom: "uscrt" }],
    gas: "80000",
  },
}
async function main() {
  let clients = []

  //create a client for each account
  for (let account of accounts) {
    const signingPen = await Secp256k1Pen.fromMnemonic(account.mnemonic);
    const myWalletAddress = pubkeyToAddress(
      encodeSecp256k1Pubkey(signingPen.pubkey),
      "secret"
    );
    const txEncryptionSeed = EnigmaUtils.GenerateNewSeed();
    const client = new SigningCosmWasmClient(
      httpUrl,
      myWalletAddress,
      (signBytes) => signingPen.sign(signBytes),
      txEncryptionSeed, customFees
    );
    const acc = await client.getAccount()
    console.log(`wallet=${myWalletAddress}`);
    account.balance = acc.balance[0].amount;

    console.log(`balance=${myWalletAddress}, ${account.balance}`);
    clients.push(client);

    break
  }
  let client = clients[0];

  const wasm = fs.readFileSync(__dirname + "/../../contract.wasm");
  const uploadReceipt = await client.upload(wasm, {})
  console.info(`Upload succeeded. Receipt: ${JSON.stringify(uploadReceipt)}`);
  const codeId = uploadReceipt.codeId;
//  const codeId = 1;

  const memo = "raffle50";
  const initMsg = {"seed": "i love cupcakes too"}
  const { contractAddress } = await client.instantiate(codeId, initMsg, memo);
  console.info(`Contract instantiated at ${contractAddress}`);

//  const contractAddress = 'secret1ry6mddtfz5s9hx5nrhpc2ap0e6835fxklw0j5l';

  for (var i = 0; i < 1; i++) {
    let result = await clients[i].queryContractSmart(contractAddress, { config: {  } });

    // first account mostly wins
    const amount = i == 0 ? 2000000 : 1000000
    const denom = "uscrt";
    const stake = [coin(amount, denom)];

    result = await clients[i].execute(contractAddress,
      {
        join: {phrase: "we love cupcakes" + i}
      }, "", stake);
    result = await clients[i].queryContractSmart(contractAddress, { config: {  } });
    console.log(result.entries)
  }

  console.log('fomo round 2')
  for (var i = 0; i < 1; i++) {
    let result = await clients[i].queryContractSmart(contractAddress, { config: {  } });

    const amount = i == 0 ? 2000000 : 1000000
    const denom = "uscrt";
    const stake = [coin(amount, denom)];

    result = await clients[i].execute(contractAddress,
      {
        join: {phrase: "om nom nom cupcakes" + i}
      }, "", stake);
    result = await clients[i].queryContractSmart(contractAddress, { config: {  } });
    console.log(result.entries)
  }
  console.log('end round 2')
  let result = await client.execute(contractAddress, {
    end_lottery: {winner_to_select: 1}
  });
  result = await client.queryContractSmart(contractAddress, { config: {  } });
  console.log(result.entries)
  result = await client.queryContractSmart(contractAddress, { winner: {  } });
  console.log(result.winner)

  for (var i = 0; i < 1; i++) {
      const acc = await clients[i].getAccount()
      account.balance = acc.balance[0].amount;

      console.log(`wallet=${acc.address}, balance=${account.balance}`);
    }
}

main().then(
  () => {
    console.info("contract deployed.");
    process.exit(0);
  },
  error => {
    console.error(error);
    process.exit(1);
  },
);

