prompt() {
    echo -n "$1
> "
    read -r prompt_result
    echo ""
}

describe_evm_tx() {
    echo "EVM tx with id $1 submited"
    echo "You could find it in explorer https://sepolia.etherscan.io/tx/$1"
    echo ""
}

describe_miden_tx() {
    echo "Miden tx with id $1 submited"
    echo "You could find it in explorer https://testnet.midenscan.com/tx/$1"
    echo ""
}

SEPOLIA_RPC_URL="https://ethereum-sepolia-rpc.publicnode.com"
EVM_MINIMUM_ALLOWED_BALANCE=10000000000000000
USDC_EVM_ADDRESS="0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"
USDC_MIDEN_ADDRESS="0x4de3bc8d67731a2067af0fcc7a2e34"
BRIDGE_EVM_ADDRESS="0x0b03df1D4B3884b8987254D0C990342B571183AF"

evm_to_miden() {
    if [[ ! -e "miden-client.toml" ]]; then
        miden-bridge init --network testnet
    fi

    prompt "Put your evm private key"
    privatekey=$prompt_result
    address=$(cast wallet address --private-key $privatekey)
    balance=$(cast balance $address --rpc-url $SEPOLIA_RPC_URL)
    formated_balance=$(cast to-unit $balance ether)

    usdc_balance=$(cast balance $address --rpc-url $SEPOLIA_RPC_URL --erc20 $USDC_EVM_ADDRESS | awk '{print $1}')
    formated_usdc_balance=$(echo "scale=2 ; $usdc_balance / 1000000" | bc )
    echo "Working with evm address: $address with $formated_balance ETH and $formated_usdc_balance USDC"

    if (( $balance < $EVM_MINIMUM_ALLOWED_BALANCE )); then
        echo "Account ETH balance too low"
        exit 1
    fi
    
    receiver=$(miden-bridge account -d 2>/dev/null | grep -o "0x[0-9a-f]*")

    if [[ -z $receiver ]]; then
        miden-bridge new-wallet &>/dev/null
        receiver=$(miden-bridge account -d 2>/dev/null | grep -o "0x[0-9a-f]*")
    fi

    echo "Miden receiver address: $receiver"

    prompt "How much USDC you want to transfer?"
    formated_amount=$prompt_result
    amount=$(( $formated_amount * 1000000 ))

    if (( $usdc_balance < $amount )); then
        echo "Too low usdc balance"
        exit 1
    fi

    echo "Approval transaction generation"
    tx_id=$(cast publish --async -r $SEPOLIA_RPC_URL "$(cast mktx -r $SEPOLIA_RPC_URL --private-key $privatekey -f $address $USDC_EVM_ADDRESS "approve(address,uint256)" $BRIDGE_EVM_ADDRESS $amount)")
    describe_evm_tx $tx_id

    sleep 40

    recipient_response=$(miden-bridge recipient -a $receiver)
    echo $recipient_response
    recipient=$(echo "$recipient_response" | grep -o "0x[0-9a-f]*" | head -n 1)
    serial_number=$(echo "$recipient_response" | grep -o "0x[0-9a-f]*" | tail -n 1)

    echo "Bridge transaction generation"
    tx_id=$(cast publish --async -r $SEPOLIA_RPC_URL "$(cast mktx -r $SEPOLIA_RPC_URL --private-key $privatekey -f $address $BRIDGE_EVM_ADDRESS "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" $USDC_EVM_ADDRESS $amount 9966 0x0000000000000000000000000000000000000000 0x0000000000000000000000000000000000000000 $recipient false)")
    describe_evm_tx $tx_id

    for i in {1..5}; do
        echo "Waiting for relayer..."
        sleep 90

        reconstruct_output=$(miden-bridge reconstruct --serial-number $serial_number --account-id $receiver --asset-amount $amount --faucet-id $USDC_MIDEN_ADDRESS 2>/dev/null)

        if [[ "$?" == "0" ]]; then
            reconstructed_note_id=$(echo "$reconstruct_output" | grep -o "Reconstructed note id: 0x[0-9a-f]*" | grep -o "0x[0-9a-f]*")
            miden-bridge consume-notes -a $receiver $reconstructed_note_id

            break
        else
            echo "Bridging still in progress"
        fi
    done

    echo "Bridging to miden finished!"
}

mixing() {
    if [[ ! -e "miden-client.toml" ]]; then
        miden-bridge init --network testnet
    fi

    prompt "Put your evm private key"
    privatekey=$prompt_result
    address=$(cast wallet address --private-key $privatekey)
    balance=$(cast balance $address --rpc-url $SEPOLIA_RPC_URL)
    formated_balance=$(cast to-unit $balance ether)

    usdc_balance=$(cast balance $address --rpc-url $SEPOLIA_RPC_URL --erc20 $USDC_EVM_ADDRESS | awk '{print $1}')
    formated_usdc_balance=$(echo "scale=2 ; $usdc_balance / 1000000" | bc )
    echo "Working with evm address: $address with $formated_balance ETH and $formated_usdc_balance USDC"

    if (( $balance < $EVM_MINIMUM_ALLOWED_BALANCE )); then
        echo "Account ETH balance too low"
        exit 1
    fi

    prompt "Put the receiver address"
    receiver_address=$prompt_result

    receiver_before_usdc_balance=$(cast balance $receiver_address --rpc-url $SEPOLIA_RPC_URL --erc20 $USDC_EVM_ADDRESS | awk '{print $1}')
    formated_receiver_before_usdc_balance=$(echo "scale=2 ; $receiver_before_usdc_balance / 1000000" | bc )
    echo "Receiver evm address: $receiver_address with $formated_receiver_before_usdc_balance USDC"

    prompt "How much USDC you want to transfer?"
    formated_amount=$prompt_result
    amount=$(( $formated_amount * 1000000 ))

    if (( $usdc_balance < $amount )); then
        echo "Too low usdc balance"
        exit 1
    fi

    echo "Approval transaction generation"
    tx_id=$(cast publish --async -r $SEPOLIA_RPC_URL "$(cast mktx -r $SEPOLIA_RPC_URL --private-key $privatekey -f $address $USDC_EVM_ADDRESS "approve(address,uint256)" $BRIDGE_EVM_ADDRESS $amount)")
    describe_evm_tx $tx_id

    sleep 40

    recipient_response=$(miden-bridge recipient --note-type crosschain --dest-chain 11155111 --dest-address $receiver_address)
    echo $recipient_response
    bridge_note_serial_number=$(echo "$recipient_response" | grep -o "0x[0-9a-f]*" | head -n 1)
    recipient=$(echo "$recipient_response" | grep -o "0x[0-9a-f]*" | head -n 2 | tail -n 1)
    serial_number=$(echo "$recipient_response" | grep -o "0x[0-9a-f]*" | tail -n 1)

    echo "Bridge transaction generation"
    tx_id=$(cast publish --async -r $SEPOLIA_RPC_URL "$(cast mktx -r $SEPOLIA_RPC_URL --private-key $privatekey -f $address $BRIDGE_EVM_ADDRESS "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" $USDC_EVM_ADDRESS $amount 9966 0x0000000000000000000000000000000000000000 0x0000000000000000000000000000000000000000 $recipient false)")
    describe_evm_tx $tx_id

    for i in {1..5}; do
        echo "Waiting for relayer..."
        sleep 90

        reconstruct_output=$(miden-bridge mix --serial-number $serial_number --bridge-serial-number $bridge_note_serial_number --dest-chain 11155111 --dest-address $receiver_address --asset-amount $amount --faucet-id $USDC_MIDEN_ADDRESS 2>/dev/null)

        if [[ "$?" == "0" ]]; then
            reconstructed_note_id=$(echo "$reconstruct_output" | grep -o "Reconstructed note id: 0x[0-9a-f]*" | grep -o "0x[0-9a-f]*")
            tx_id=$(echo "$reconstruct_output" | grep -o "Generated tx id: 0x[0-9a-f]*" | grep -o "0x[0-9a-f]*")
            describe_miden_tx $tx_id

            sleep 90

            receiver_after_usdc_balance=$(cast balance $receiver_address --rpc-url $SEPOLIA_RPC_URL --erc20 $USDC_EVM_ADDRESS | awk '{print $1}')
            formated_receiver_after_usdc_balance=$(echo "scale=2 ; $receiver_after_usdc_balance / 1000000" | bc )
            echo "Receiver evm address: $receiver_address with $formated_receiver_after_usdc_balance USDC"

            break
        else
            echo "Mixing still in progress"
        fi
    done

    echo "Mixing finished!"
}

prompt "Choose the bridge direction:
    1) EVM->Miden
    2) Miden->EVM
    3) Mixing"

direction=$prompt_result

if [[ "$direction" == "1" ]]; then
    evm_to_miden
elif [[ "$direction" == "2" ]]; then
    echo "Miden to EVM"
    echo "Not implemented yet!"
    exit 1
elif [[ "$direction" == "3" ]]; then
    mixing
else
    echo "Unknown option. Acceptable options: 1|2|3"
    exit 1
fi
