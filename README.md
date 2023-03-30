# CW 721 Proxy

1. `packages/cw721-proxy` An interface for proxying cw721 send messages.
2. `contracts/cw721-rate-limited-proxy` An implementation of this
   proxy interface to rate limit incoming cw721 send messages.
3. `packages/cw-rate-limiter` package for rate limiting in CosmWasm
   contracts.
4. `packages/cw721-proxy-derive` Procedural macros for deriving the
   proxy receiver message types on an existing enum.
5. `packages/cw721-whitelist` package for whitelisting in CosmWasm
   contracts.
6. `contracts/cw721-code-id-proxy` A proxy with a code id whitelist
   being eligible to `send_nft`s to origin contract.
7. `contracts/cw721-sender-proxy` A proxy with a sender (cw721) whitelist
   being eligible to `send_nft`s to origin contract.
8. `contracts/ics721-channel-proxy` An ICS721 specific proxy with a channel whitelist
   being eligible to `send_nft`s to origin contract. Proxy expects to receive an
   [ICS721 IbcOutgoingMsg](https://github.com/public-awesome/ics721/blob/main/contracts/cw-ics721-bridge/src/msg.rs#L84-L95).
9. `contracts/ics721-sender-channel-proxy` An ICS721 specific proxy with a sender and channels whitelist
   being eligible to `send_nft`s to origin contract. Proxy expects to receive an
   [ICS721 IbcOutgoingMsg](https://github.com/public-awesome/ics721/blob/main/contracts/cw-ics721-bridge/src/msg.rs#L84-L95).
