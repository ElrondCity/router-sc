#--------TECHNICAL--------

ADDRESS="erd1qqqqqqqqqqqqqpgqclgfyk92zkpau8xes8tnctq9herpyj6ll5nscq2v4u"
OWNER_ADDRESS="erd1hm5dkcrp6xg3573ndl7n4a3x97u4dysa3ef6e7lgee8j3vz5l5nsa34h04"
PRIVATE_KEY=(--keyfile=erd1hm5dkcrp6xg3573ndl7n4a3x97u4dysa3ef6e7lgee8j3vz5l5nsa34h04.json --passfile=.passfile)
PASSFILE=--passfile=.passfile # Ignore that
PROXY=https://devnet-api.elrond.com
CHAIN_ID=D

TOKEN_IDENTIFIER="EVILLE-a7976c" # Token identifier
TOKEN_NONCE=0 # 0 for fungible tokens

deploy() {
    erdpy --verbose contract deploy --bytecode output/router-sc.wasm --recall-nonce ${PRIVATE_KEY} --gas-limit=500000000 --proxy=${PROXY} --chain=${CHAIN_ID} --metadata-payable --metadata-payable-by-sc --send --outfile="deploy.interaction.json" || return

    TRANSACTION=$(erdpy data parse --file="deploy.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=router-sc-address-devnet --value=${ADDRESS}
    erdpy data store --key=router-sc-deployTransaction-devnet --value=${TRANSACTION}

}

upgrade() {
   echo "Upgrading Smart Contract address: ${ADDRESS}"
   erdpy --verbose contract upgrade ${ADDRESS} --bytecode output/router-sc.wasm --recall-nonce ${PRIVATE_KEY} --gas-limit=500000000 --proxy=${PROXY} --chain=${CHAIN_ID} --send

}

addDistribution() {
    distributionAddress=${1}
    distributionPercentage=${2}

    echo "Adding distribution address: ${distributionAddress} with percentage: ${distributionPercentage}"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce ${PRIVATE_KEY} \
            --gas-limit=50000000 \
            --proxy=${PROXY} --chain=${CHAIN_ID} \
            --function="addDistribution" \
            --arguments ${distributionAddress} ${distributionPercentage} \
            --send
    echo $?

}

removeDistribution() {
    distributionAddress=${1}

    echo "Removing distribution address: ${distributionAddress}"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce ${PRIVATE_KEY} \
            --gas-limit=50000000 \
            --proxy=${PROXY} --chain=${CHAIN_ID} \
            --function="removeDistribution" \
            --arguments ${distributionAddress} \
            --send
    echo $?
}

setToken() {
    tokenId="0x$(echo -n ${1} | xxd -p -u | tr -d '\n')"

    echo "Setting token: ${tokenId}"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce ${PRIVATE_KEY} \
            --gas-limit=50000000 \
            --proxy=${PROXY} --chain=${CHAIN_ID} \
            --function="addToken" \
            --arguments ${tokenId} \
            --send
    echo $?
}

distribute() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce ${PRIVATE_KEY} \
            --gas-limit=50000000 \
            --proxy=${PROXY} --chain=${CHAIN_ID} \
            --function="distribute" \
            --send
    echo $?
}