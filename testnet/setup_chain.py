import json
import os
import asyncio
import secrets
import shutil
import signal
import subprocess
import sys
import threading
import time
import web3
from eth_account import Account
from dotenv import load_dotenv


def gen_key_address_pair():
    private_key = "0x" + secrets.token_hex(32)
    account_1 = Account.from_key(private_key).address
    return account_1, private_key


def capture_output(process):
    while True:
        output = process.stdout.readline()
        if output:
            print(output.strip())
        else:
            break

    process.communicate()


async def main():
    load_dotenv()
    chain_num = 987789
    tmp_dir = 'tmp/tmp'
    chain_dir = f"{tmp_dir}/chain{chain_num}"
    genesis_file = f"{tmp_dir}/genesis{chain_num}.json"
    addresses_file = f"{tmp_dir}/addresses{chain_num}.json"

    # get private key from env
    main_account = os.environ['MAIN_ACCOUNT_PRIVATE_KEY']
    signer_account = os.environ['SIGNER_ACCOUNT_PRIVATE_KEY']
    keystore_password = os.environ['SIGNER_ACCOUNT_KEYSTORE_PASSWORD']
    keep_running = int(os.environ['KEEP_RUNNING']) == 1

    (address1, private_key1) = (
        Account.from_key(main_account).address,
        main_account)

    print(f"Loaded main account: {address1}")

    (signer_address, signer_private_key) = (
        Account.from_key(signer_account).address,
        signer_account)

    deploy_contracts = False
    if not os.path.exists(tmp_dir):
        deploy_contracts = True

        os.makedirs(tmp_dir)

        print(f"Loaded signer account: {signer_address}")

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
                    "period": 5,
                    "epoch": 0
                }
            },
            "difficulty": "1",
            "gasLimit": "30000000",
            # Signer address for clique
            "extradata": "0x0000000000000000000000000000000000000000000000000000000000000000"
                         + f"{signer_address}".lower().replace("0x", "")
                         + "000000000000000000000000000000000000000000000000000000000000000000"
                         + "0000000000000000000000000000000000000000000000000000000000000000",
            "alloc": {
                address1: {"balance": '1000000000000000000000000000'},
                "0x001066290077e38f222cc6009c0c7a91d5192303": {"balance": '1000000000000000000000000000'},
                "0x00203654961340f35726ce63eb4bf6912a62022e": {"balance": '1000000000000000000000000000'},
                "0x003047f2f6b0c66a07e60f149276211dc2ff7489": {"balance": '1000000000000000000000000000'},
                "0x0040bcefcb706641104a9feb95ad59830c30671b": {"balance": '1000000000000000000000000000'},
                "0x005014c6eea59620aea92298f8af003bed130ad0": {"balance": '1000000000000000000000000000'},
                "0x0060967f2181b0e496b1a1a0389b9c2f3d8dc2a9": {"balance": '1000000000000000000000000000'},
                "0x0070619c46c4b2738c0ce73d63bbe061391fd80f": {"balance": '1000000000000000000000000000'},
                "0x0080dc12044e18c8f53c7ccced0ef776d4b3bfd8": {"balance": '1000000000000000000000000000'},
                "0x0090167b580b0b3c79e24f6b919762b9e5cf0a05": {"balance": '1000000000000000000000000000'},
                "0x0100652743be2dc18637c05bf5e49cfc87f30243": {"balance": '1000000000000000000000000000'}
            }
        }

        with open(f'{genesis_file}', 'w') as f:
            json.dump(genesis, f, indent=4)

        os.system(f'geth --datadir {chain_dir} init {genesis_file}')

        keystore = Account.encrypt(signer_account, keystore_password)

        with open(f'{chain_dir}/keystore/testnet_key', 'w') as f:
            f.write(json.dumps(keystore, indent=4))
        with open(f'{chain_dir}/keystore/testnet_key_pass.txt', 'w') as f:
            f.write(keystore_password)

    # clique signer/miner settings
    miner_settings = f"--mine --allow-insecure-unlock --unlock {signer_address} --password {chain_dir}/keystore/testnet_key_pass.txt"
    geth_command = f'geth --datadir={chain_dir} ' \
                   f'--nodiscover ' \
                   f'--syncmode=full ' \
                   f'--gcmode=archive ' \
                   f'--http ' \
                   f'--http.addr=0.0.0.0 ' \
                   f'--http.vhosts=* ' \
                   f'--http.corsdomain=* ' \
                   f'--http.api=eth,net,web3,txpool ' \
                   f'--rpc.txfeecap=1000 ' \
                   f'--networkid={chain_num} {miner_settings}'
    print(geth_command)
    # os.system(geth_command)

    geth_command_split = geth_command.split(' ')
    process = subprocess.Popen(geth_command_split, stdout=subprocess.PIPE)
    thread = threading.Thread(target=capture_output, args=(process,))
    thread.start()
    # give the process some time to start http server
    await asyncio.sleep(2)

    if deploy_contracts:
        # deploy contracts
        os.chdir("contracts-web3-create2")
        os.system("npm run deploy_dev")

    print("Blockchain is ready for testing")

    if not keep_running:
        print("Testing complete. Shutdown blockchain")
        process.kill()
        thread.join()
        print(geth_command)
    else:
        while True:
            await asyncio.sleep(2)


if __name__ == "__main__":
    if sys.platform == 'win32':
        # Set the policy to prevent "Event loop is closed" error on Windows - https://github.com/encode/httpx/issues/914
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

    asyncio.run(main())
