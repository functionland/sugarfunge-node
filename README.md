[//]: # (SBP-M1 review: needs updating to Functionland, with link to forked repo)
![SugarFunge](/docs/sf-name.png)
# Substrate-based SugarFunge Node

The SugarFunge blockchain powers the SugarFunge Protocol. A protocol for companies looking to model their business logic in a privately-owned economy without the complexity of building and maintaining their own blockchain infrastructure. Empowering businesses with digital and physical assets with minting, crafting and trading mechanics.

Read more about [Owned Economies](https://github.com/SugarFunge/OwnedEconomies).


## Local Testnet
> **Important**: to be able to use all [sugarfunge-api](https://github.com/functionland/sugarfunge-api) endpoints without problem you must run at least two validators

<br/>

1st Validator:
```bash
./target/release/sugarfunge-node key insert --base-path=/var/lib/.sugarfunge-node/data/node01 --keystore-path=/var/lib/.sugarfunge-node/keys/node01 --chain customSpecRaw.json --scheme Sr25519 --suri "word1 ... word12" --password-filename "/var/lib/.sugarfunge-node/passwords/password1.txt" --key-type aura

./target/release/sugarfunge-node key insert --base-path=/var/lib/.sugarfunge-node/data/node01 --keystore-path=/var/lib/.sugarfunge-node/keys/node01 --chain customSpecRaw.json --scheme Ed25519 --suri "word1 ... word12" --password-filename "/var/lib/.sugarfunge-node/passwords/password1.txt" --key-type gran

cargo run --release -- --chain ./customSpecRaw.json --enable-offchain-indexing true --base-path=/var/lib/.sugarfunge-node/data/node01 --keystore-path=/var/lib/.sugarfunge-node/keys/node01 --port=30334 --rpc-port 9944 --rpc-cors=all --rpc-methods=Unsafe --rpc-external --validator --name MyNode01 --password-filename="/var/lib/.sugarfunge-node/passwords/password1.txt" --node-key=peerID_secret_key
```
2nd Validator:
``` bash
./target/release/sugarfunge-node key insert --base-path=/var/lib/.sugarfunge-node/data/node02 --keystore-path=/var/lib/.sugarfunge-node/keys/node02 --chain customSpecRaw.json --scheme Sr25519 --suri "word1 ... word12" --password-filename "/var/lib/.sugarfunge-node/passwords/password2.txt" --key-type aura

./target/release/sugarfunge-node key insert --base-path=/var/lib/.sugarfunge-node/data/node02 --keystore-path=/var/lib/.sugarfunge-node/keys/node02 --chain customSpecRaw.json --scheme Ed25519 --suri "word1 ... word12" --password-filename "/var/lib/.sugarfunge-node/passwords/password2.txt" --key-type gran

cargo run --release -- --chain ./customSpecRaw.json --enable-offchain-indexing true --base-path=/var/lib/.sugarfunge-node/data/node02 --keystore-path=/var/lib/.sugarfunge-node/keys/node02 --port=30335 --rpc-port 9945 --rpc-cors=all --rpc-methods=Unsafe --rpc-external --validator --name MyNode02 --password-filename="/var/lib/.sugarfunge-node/passwords/password2.txt" --node-key=peerID_secret_key --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWBeXV65svCyknCvG1yLxXVFwRxzBLqvBJnUF6W84BLugv
```
[//]: # (SBP-M1 review: insufficient documentation)
[//]: # (SBP-M1 review: limited tests)
[//]: # (SBP-M1 review: LICENSE file empty)
[//]: # (SBP-M1 review: `docker run functionland/node:release` fails with `/run_node.sh: 11: wait: Illegal option -n`)