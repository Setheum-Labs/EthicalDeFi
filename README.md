بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

# ZIMS Protocol - Powering Ethical DeFi Solutions.

⚠️WIP - Work In Progress, Not Production Ready!

Setheum's ECDP (Ethical Collateralied Debt Position) Protocol is a CDP Stablecoin protocol in Setheum built for the Zero-Interest stablecoin market. Inspired by MakerDAO Protocol, the Setheum ECDP protocol has zero interest rates, zero stability fees, and is fully halal and over-collateralized as well as multicollateralized. This lets the Ethical DEfi and Islamic Finance world to also participate in the industry and take part in trading and yield making strategies that are within their principles, beliefs, and customs.

### Currencies 'n Sub-Protocols

There are two (2) sub-protocols in Khalifa ECDP, these two differ in their underlying stability mechanisms as well as collateralization and redemption mechanisms, ther are:

####  Pegged Stablecoin Protocol

Pegged to a certain fiat currency such as USD, and relatively stable. The protocol is inspired by MakerDAO.
 
Curreencies:
- `SlickUSD - USSD`: Pegged 1:1 to the USD.
Confirmed Collaterals are `SEE, KHA, BTC, ETH, BNB, DOT, KSM, SETR, GRA`;

####  Unpegged Stablecoin Protocol

Not pegged to any fiat currency, and relatively stable-ish.The protocol is inspired by RAI which was inspired by MakerDAO.

It is not pegged to any particular fiat currency, but has a target price at which it is relatively stable-ish (more like the concept of the USD itself, but with the gold standard). There is also a redemption rate which changes to relatively stabilize the currency. The stablecoin is like an stablecoin index that tracks the market movement and volatility of it's mirrored asset and blends it to a smoother less volatile, slower moving price level that keeps the stablecoins relatively stable around its target price.

The stablecoins are multi-collateralized and over-collateralized. The currency's mirrored asset(which we call `scale index` or `SI`) is a basket of cryptocurrencies (eg. `SEE`, `BTC` and `KHA`, could be updated to add or remove currencies from the `SI` through governance), it tracks the average of the `SI` with a customizable index ratio(currently equal at `50:25:25` could be updated through governance). These parameters are updatable by the protocol governance in order to properly consolidate and strengthen the `SI` of the currency, thereby increasing stability.

Curreencies:
- `Setter - SETR`: The `SI` of `50:30:20` on `SEE:KHA:BTC`;
- `Golden Ratio - GRA`: The `SI` of `20:20:20:20:20` on `SEE:KHA:BTC:ETH:BNB`;

 One implementation of the protocol is the `Setter - SETR` and the `Golden Ratio - GRA`: The `SETR` and `GRA` are `unpegged`,  The Setter is inspired by MakerDAO and RAI Stablecoin.

## LICENSE
The primary license for ECDP is the Business Source License 1.1 (BUSL-1.1), see [LICENSE](https://github.com/Khalifa-Blockchain/ECDP/blob/main/LICENSE.md).
