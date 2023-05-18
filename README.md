# HERO

## Code In Depth Documentations
For more detailed documentations about the code itself, [click here](./docs)

Hero is Polkadot parachain on which we can deploy and use solidity smart contracts.


## Running a Hero Collator
1.) First things first is to [install docker](https://www.docker.com/products/docker-desktop) for your particular platform.

2.) clone Hero repository `git clone https://github.com/PAIDNetwork/hero.git && cd hero`

3.) build docker base `docker build -f Dockerfile.base -t paidnetwork/rust .`

4.) build collator image `docker build -t paidnetwork/collator .`

5.) Open the following ports `30344` `40334` `6968` `8845` `9978` if you would like to connect to your collator remotely open port `80`

6.) run the collator and frontend `docker-compose -f scripts/docker-compose-collator.yaml up -d`

You now should beable to view your collator via [polkadotjs](https://polkadot.js.org/apps/#/explorer) switch your network and under
the `Develoment tab` click `Custom` and paste `ws://localhost:8845` then click `Switch` at the top.

## Hero Dev Environment Docker

1.) Clone hero `git clone https://github.com/PAIDNetwork/hero.git && cd hero`

2.) build docker base `docker build -f Dockerfile.base -t paidnetwork/rust .`

3.) build collator image `docker build -t paidnetwork/collator .`

4.) Clone Polkadot `cd $HOME && git clone https://github.com/PAIDNetwork/polkadot.git`

5.) checkout correct polkadot version `cd polkadot && git checkout release-v0.9.18`

6.) build relay chain image `docker build -f Dockerfile.relay -t paidnetwork/relay .`

7.) Run containers via docker compose `cd $HOME/hero && docker-compose -f docker-compose-relay.yaml up -d`

You now should beable to view your collator via [polkadotjs](https://polkadot.js.org/apps/#/explorer) switch your network and under
the `Develoment tab` click `Custom` and paste `ws://localhost:8844` then click `Switch` at the top. For the relay chain
connect to `ws://localhost:9944`

Note** the chainspec which is being used for the parachain is "rococo-local" please reference hero/node/src/chain_spec.rs for more
details on the exact configuration. If you would like to change simply make an edit to the docker-compose or upgrade the chain_spec.rs
i.e.(Sudo key)


Follow instructions on [Cumulus Tutorial](https://docs.substrate.io/tutorials/v3/cumulus/connect-parachain/) for registering your parachain on the local relay chain. Remember the paraid is 2000

# Installation Hero Development
The first thing you will need to do is prepare the system for Substrate development.

Follow this [link](https://docs.substrate.io/v3/getting-started/installation/) for development environment setup.

## Download and install Polkadot
* Clone repo:
```bash
git clone https://github.com/PAIDNetwork/polkadot.git
```
* Change to polkadot directory:
```bash
cd polkadot
```
* Checkout the latest working commits:
```bash
git checkout release-v0.9.27
```
* Build the relay chain:
```bash
cargo build --release
```
* Check if the build succeeded:
```bash
./target/release/polkadot --help
```

## Download and install hero
* Clone repo:
```bash
git clone git@github.com:PAIDNetwork/hero.git
```
* Change directory:
```bash
cd hero
```
* Build hero parachain collator:
```bash
cargo build --release
```
* Check if build succeeded:
```bash
./target/release/hero --help
```
___

__WARNING:__

If you are developing Hero, it is __imperative__ that you use `cargo build --release` first and be __wary__ of `cargo check`.
__Type checking depends__ on the __WASM binary__ that is built before compile time _(in a `build.rs`)_.
Attempting to `cargo check` from a clean slate __will fail__ 
 - _( eg: this will fail `cargo clean && cargo update && cargo check` )_

If you are having issues with updating dependancies (a common one is having 2 versions of `sp-io`), 
consider clearing the cargo's cache and rebuilding clean.

* First have a utility to delete cargo's cache, I recommend `cargo-cache`.
```
cargo install cargo-cache
```
* Then clean the cache, clean the target directory, update the `Cargo.lock`, and rebuild
```
cargo cache -a && cargo clean && cargo update && cargo build --release
```

___

## Start Local __Relay Chain Validators__
* Open 2 Seperate Terminals via <kbd>cmd+t</kbd> (for Mac) & <kbd>ctrl+t</kbd> (for Win, Linux machine) in the `PAIDNetwork/polkadot` cloned repo.

<!-- reference ( https://github.com/paritytech/cumulus ) -->

## Generate a __raw relay chain spec__ _(polkadot repo)_
```bash
./target/release/polkadot build-spec --chain rococo-local \
--disable-default-bootnode --raw > rococo-local-cfde.json
```

## Validators
* we start 2 validators to run

### Alice
* In __terminal-1__ _(polkadot repo)_
```bash
./target/release/polkadot --alice --tmp --validator \
--chain rococo-local-cfde.json \
--port 30333 --ws-port 9944
```
<!--
# (changes from 0.9.18 to 0.9.19 ::
# --base-path <path> ==> --tmp
# --chain rococo-local ==> --chain rococo-local-cfde.json (it gets explicitly generated before use)
#
# done to avoid epoch change issues (https://ink.substrate.io/getting-started/troubleshooting/)
# )
-->

### Bob
* In __terminal-2__ _(polkadot repo)_
```bash
./target/release/polkadot --bob --tmp --validator \
--chain rococo-local-cfde.json \
--port 30334 --ws-port 9945
```

* __If the above fails__ you can supply `--bootnodes <path>` passing identity of Alice to Bob explicitly

Copy Alice's __Local Node Identity__ which looks something like this in Alice's __terminal-1__ output:
```bash
Local node identity is: 12D3KooWEwPTb5sQamy43HuqTc9doUyVwUZs7dNUrJEPJ48pc9Yr
```

and paste in place of `<Alices local node identity from above>` for Bob in __terminal-2__
```bash
./target/release/polkadot --bob --tmp --validator \
--chain rococo-local-cfde.json \
--port 30334 --ws-port 9945 \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/<Alices local node identity from above>
```
<!-- bootnodes should be last to make editing the local node identity at the end easy -->

## Copy __relay chain spec__ for parachain
It was generated earlier inside of the Polkadot repo.
Copy it into __root__ of the `hero` repo. _(example in `bash` called from polkadot repo)_
```bash
cp rococo-local-cfde.json ../hero/rococo-local-cfde.json
```

____

You can now __leave the relay chain to run__.
Continue by setting up a __third terminal__, running in the `hero` repo.
____
## Build __collator spec__ and __raw chain spec__
## => Obtain __wasm runtime__ validation function and parachain __genesis state__
* Required to pass to `sudoScheduleParaInitialize` later for registration _(hero repo)_:
* __terminal-3__ _(hero repo)_
<!-- Assumes that `rococo-local` is in `node/chan_spec.rs` as the relay you registered with -->
```bash
./target/release/hero build-spec \
--disable-default-bootnode > rococo-local-parachain-plain.json \
&& \
./target/release/hero build-spec --chain rococo-local-parachain-plain.json \
--raw --disable-default-bootnode > parachain-raw.json \
&& \
./target/release/hero export-genesis-wasm \
--chain parachain-raw.json > para-wasm \
&& \
./target/release/hero export-genesis-state \
--chain parachain-raw.json > para-genesis
```


## Start __parachain collator__

* Start Hero Parachain in __terminal-3__ _(hero repo)_:
```bash
./target/release/hero --alice \
--tmp --collator --force-authoring \
--chain parachain-raw.json \
--port 40333     --ws-port 8844 \
--rpc-port 6969  --rpc-cors all \
-- \
--execution wasm \
--chain rococo-local-cfde.json \
--port 30343     --ws-port 9977
```
<!--
* optionaly add another collator _Bob (Collator 2)_
```bash
./target/release/hero --collator --bob --force-authoring --tmp \
--port 40336 --ws-port 9947 \
-- \
--execution wasm --chain rococo-local-cfde.json --port 30336
```
-->

<!--
* optionally add a _Full Node_
```
./target/release/hero --tmp \
--port 40337 --ws-port 9948 \
-- \
--execution wasm --chain rococo-local-cfde.json --port 30337
```
-->

____
From this point we can __leave all the terminals running__, proceede to the __web browser__
__( https://polkadot.js.org/apps/#/explorer )__
____

## Register __parathread/parachain__

* Start __polkadot.js__ apps:
```
https://polkadot.js.org/apps/#/explorer
#click drop down arrow on the top left by "Rococo Local Testnet"
#select Local Node 127.0.0.1:9944
```

* Navigate the ui to register a __parathread id__:
```
Network -> Parachains -> Parathreads -> ParaId(with a plus sign)
# click sign and submit, note the id number (the first currently defaults to 2000)
```
<!--
for reference ( https://docs.substrate.io/how-to-guides/v3/parachains/connect/ )
-->

* Register the parathread id on your relay chain

On __polkadot.js__ app do the following:
```
Developer -> Sudo -> (under submit the following change) paraSudoWrapper ->
(on the right drop down) sudoScheduleParaInitialize(id, genesis)
# assign the id that was registered earlier
# click file upload for both genesisHead and validationCode fields
# upload para-genesis && para-wasm in their respective fields.
# change parachain bool to 'yes'
```
* You will need to wait 2 minutes for your parachain to be accepted by the relay chain validators.
  check this in:
```
Network -> Parachains -> Overview
```

# Complete!
You should see that your collator is collating and you can swap to interact with your hero collator
on the top left corner!!
Create a custom endpoint:
```
ws://127.0.0.1:8844
```
# smart-agreement-chain
