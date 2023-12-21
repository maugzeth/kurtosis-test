from web3 import Web3

TEST_ACCOUNT_PRIVATE_KEY = "bcdf20249abf0ed6d944c0288fad489e33f66b3960d9e6229c1cd214ed3bbe31"
NODE_RPC_URL = "http://127.0.0.1:60558"

nonce = 0

# Connect to RPC Client
w3 = Web3(Web3.HTTPProvider(NODE_RPC_URL))
if w3.is_connected():
    print("[*] Connected to RPC Client.")

# Fetch latest block info (for debugging)
latest_block = w3.eth.get_block('latest')
print(f"[*] Latest Block: {latest_block.number} ({len(latest_block.transactions)})")

# Format sample transaction 
tx_params = dict(
    nonce=nonce,
    # "from": "0xf93Ee4Cf8c6c40b329b0c0626F28333c132CF241",
    to="0x8943545177806ED17B9F23F0a21ee5948eCaa776",
    gas=100000,
    gasPrice="0x9184e72a000",
    value=1,
)

# Sign transaction
signed_tx = w3.eth.account.sign_transaction(tx_params, TEST_ACCOUNT_PRIVATE_KEY)
print(f"[*] Signed Transaction: {Web3.to_hex(signed_tx.hash)}")

# Send Transaction
sent_tx = w3.eth.send_raw_transaction(signed_tx.rawTransaction)
print(f"[*] Sent Transaction: {Web3.to_hex(sent_tx)}")

# Increment nonce on success (prevent having to do it manually)
nonce += 1

print("[*] Done.")
