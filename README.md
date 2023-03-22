# CW 721 Proxy

1. `packages/cw721-proxy` An interface for proxying cw721 send messages.
3. `contracts/cw721-rate-limited-proxy` An implementation of this
   proxy interface to rate limit incoming cw721 send messages.
2. `packages/cw-rate-limiter` package for rate limiting in CosmWasm
   contracts.
4. `packages/cw721-proxy-derive` Procedural macros for deriving the
   proxy receiver message types on an existing enum.
5. `contracts/cw721-sender-whitelist-proxy` A proxy with a (sender) whitelist
   being eligible to `send_nft`s to origin contract.
6. `packages/cw721-sender-whitelist` package for whitelisting in
   CosmWasm contracts.
