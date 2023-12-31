# CW721 Incoming Proxy

This incoming proxy allows whitelisting channels. It validates incoming `IbcPacket` and checks from which channel packet is coming from.
In case channel is not whitelisted an `UnauthorizedSourceChannel` is thrown.