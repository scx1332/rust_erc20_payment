import json
import os
import asyncio
import secrets
import shutil
import subprocess
import sys
import time
import web3
from eth_account import Account


def gen_key_address_pair():
    private_key = "0x" + secrets.token_hex(32)
    account_1 = Account.from_key(private_key).address
    return account_1, private_key


async def main():
    chain_num = 77
    tmp_dir = 'tmp'
    chain_dir = f"{tmp_dir}/chain{chain_num}"
    genesis_file = f"{tmp_dir}/genesis{chain_num}.json"
    addresses_file = f"{tmp_dir}/addresses{chain_num}.json"

    if os.path.exists(tmp_dir):
        shutil.rmtree(tmp_dir)
        if sys.platform == 'win32':
            time.sleep(0.5)
    os.mkdir(tmp_dir)

    # (address1, private_key1) = gen_key_address_pair()
    # (address2, private_key2) = gen_key_address_pair()
    (address1, private_key1) = (
    "0x30a3b4e1a03360820f437b62e6ec6919F41a29BE", "0xee565091929f51d02c504f4c37ecc79abd5caa7a67c8917d862d4393c8992519")
    (address2, private_key2) = (
    "0xD13F0d5042542107a05b9074a6ACCdf3eE9582c0", "0xce254ba6ed14cb112a3b1bafa245f33090245cc558faa41b1b3c3eb2c97de5c8")

    genesis = {
        "config": {
            "chainId": chain_num,
            "homesteadBlock": 0,
            "eip150Block": 0,
            "eip155Block": 0,
            "eip158Block": 0,
            "byzantiumBlock": 0,
            "constantinopleBlock": 0,
            "petersburgBlock": 0,
            "istanbulBlock": 0,
            "berlinBlock": 0,
            "londonBlock": 0,
            "ArrowGlacierBlock": 0,
            "GrayGlacierBlock": 0,
            "clique": {
                "period": 0,
                "epoch": 0
            }
        },
        "difficulty": "1",
        "gasLimit": "8000000",
        # Signer address for clique
        "extradata": "0x0000000000000000000000000000000000000000000000000000000000000000"
                     "8c50eb7035c7347b48a829fb1592dc199f9a70ae"
                     "000000000000000000000000000000000000000000000000000000000000000000"
                     "0000000000000000000000000000000000000000000000000000000000000000",
        "alloc": {
            address1: {"balance": '1000000000000000000'},
            address2: {"balance": '1000000000000000000'}
        }
    }



    with open(addresses_file, 'w') as f:
        f.write(json.dumps(
            {
                "addresses": [address1, address2],
                "keys": [private_key1, private_key2]
            }, indent=4))

    with open(f'{genesis_file}', 'w') as f:
        json.dump(genesis, f, indent=4)

    os.system(f'geth --datadir {chain_dir} init {genesis_file}')

    keystore = {
        "address": "8c50eb7035c7347b48a829fb1592dc199f9a70ae",
        "crypto": {
            "cipher": "aes-128-ctr",
            "ciphertext": "0af87b3ef329c7f7a683ebe163cc4ac8ea291a3a5b10109ab447c64887f09785",
            "cipherparams": {
                "iv": "cfe5f9736076ad8264b859b8abe14525"
            },
            "kdf": "scrypt",
            "kdfparams": {
                "dklen": 32,
                "n": 262144,
                "p": 1,
                "r": 8,
                "salt": "b9123b8299803f6e99583b05f8e9d29461fd88e5351c012d2195dc726dad5cd3"
            },
            "mac": "56fc26f42daa6700cae9b2b66c141ca14de0516ffff0500e92700211685d15fa"
        },
        "id": "edab6105-a37a-4301-aa05-e07115a94ab8",
        "version": 3
    }
    with open(f'{chain_dir}/keystore/testnet_key', 'w') as f:
        f.write(json.dumps(keystore, indent=4))
    with open(f'{chain_dir}/keystore/testnet_key_pass.txt', 'w') as f:
        f.write('testnet')

    # clique signer/miner settings
    miner_settings = f"--mine --allow-insecure-unlock --unlock 0x8c50eb7035c7347b48a829fb1592dc199f9a70ae --password {chain_dir}/keystore/testnet_key_pass.txt"
    geth_command = f'geth --datadir {chain_dir} --nodiscover --http --networkid {chain_num} {miner_settings}'
    print(geth_command)
    os.system(geth_command)


if __name__ == "__main__":
    if sys.platform == 'win32':
        # Set the policy to prevent "Event loop is closed" error on Windows - https://github.com/encode/httpx/issues/914
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

    asyncio.run(main())
