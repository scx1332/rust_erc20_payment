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
    (address1, private_key1) = ("0x30a3b4e1a03360820f437b62e6ec6919F41a29BE", "0xee565091929f51d02c504f4c37ecc79abd5caa7a67c8917d862d4393c8992519")
    (address2, private_key2) = ("0xD13F0d5042542107a05b9074a6ACCdf3eE9582c0", "0xce254ba6ed14cb112a3b1bafa245f33090245cc558faa41b1b3c3eb2c97de5c8")

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
            "ethash": {}
        },
        "difficulty": "1",
        "gasLimit": "8000000",
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
            }))

    with open(f'{genesis_file}', 'w') as f:
        json.dump(genesis, f)

    os.system(f'geth --datadir {chain_dir} init {genesis_file}')

    miner_settings = "--mine --miner.threads 1 --miner.etherbase 0x0000000000000000000000000000000000000010"
    geth_command = f'geth --datadir {chain_dir} --nodiscover --http --networkid {chain_num} {miner_settings}'
    os.system(geth_command)

if __name__ == "__main__":
    if sys.platform == 'win32':
        # Set the policy to prevent "Event loop is closed" error on Windows - https://github.com/encode/httpx/issues/914
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

    asyncio.run(main())
