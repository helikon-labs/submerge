curl -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"state_getMetadata","params":["0x8f3ab3e78190e0da0741ff6630ac9641a98671494e12569109a49fd380416fff"], "id":1}' \
    https://rpc.helikon.io/polkadot \
  | jq -r .result \
  | sed 's/^0x//' \
  | xxd -r -p > ./polkadot-1004001.scale