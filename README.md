# The AgeUSD Stablecoin Protocol

AgeUSD is a novel crypto-backed algorithmic stablecoin protocol that has been created in joint partnership by the [Ergo Foundation](https://ergoplatform.org/en/foundation/), [EMURGO](https://emurgo.io/), and [IOG](https://iohk.io/en/about/) on top of the [Ergo Blockchain](https://ergoplatform.org/). This repository contains the specifications/smart contracts/off-chain code(headless dApp) of AgeUSD and holds everything required for you to deploy your own AgeUSD instance on top of the Ergo Blockchain.

AgeUSD does not rely on CDPs (collateralized debt positions) as is the current popular crypto-backed stablecoin design pattern. This was a conscious design decision made due to the fragility of CDP-based protocols in the face of sharp volatility and/or blockchain congestion. This was epitomized during [Black Thursday](https://forum.makerdao.com/t/black-thursday-response-thread/1433) where MakerDAO CDPs were triggered for liquidation due to volatility, and then sold off for $0 due to blockchain congestion which prevented others from bidding.

Thanks to its design, the scenario that happened on Black Thursday is _not possible_ for the AgeUSD protocol. Without CDPs, we do not have liquidation events nor the requirement for users to perform transactions to ensure that the liquidations actually work properly (rather than allowing a bad actor to steal funds away from the protocol). These are inherent vulnerable facets of using CDPs for minting stablecoins, and as such expose more risk to the end users.

The AgeUSD protocol has been designed to shrink the surface area where such problems may arise. The goal is trying to automate as much as possible within the math of the protocol itself rather than relying on dynamic transaction posting which is liable to breaking under blockchain congestion. This isn't to say the AgeUSD protocol solves all stablecoin problems, but it is an attempt at creating a higher assurance alternative to the current trends in the crypto-sphere.

## Table of Contents

1. [How Does The AgeUSD Protocol Work?](#how-does-the-ageusd-protocol-work-)
2. [Fees](#fees)
3. [Traversing This Repository](#traversing-this-repository)
4. [Related Works](#related-works)

## How Does The AgeUSD Protocol Work?

At its core the AgeUSD protocol is quite simple to understand. There are two kinds of parties who interact with the protocol:

1. Reserve Providers
2. AgeUSD Users

Reserve Providers submit Ergs (the native currency of Ergo) to the dApp’s reserves and by doing so mint “ReserveCoins”. Each of these ReserveCoins represent a portion of the underlying Erg reserves held in the dApp.

AgeUSD Users also submit Ergs to the dApp reserves however in their case they mint AgeUSD instead. This is only allowed by the protocol if there are sufficient reserves within the dApp (reserves are above the minimum reserve ratio). At any given moment an AgeUSD user can redeem their AgeUSD in exchange for an amount of Ergs from the reserves equal to the current exchange rate as sourced by the Erg-USD oracle pool.

Reserve Providers can only redeem their ReserveCoins for Ergs if the price of Ergs goes up (or a substantial amount of protocol fees are collected) and thus cover the value of all existing minted AgeUSD plus an extra margin. By redeeming their ReserveCoins, they profit as they receive more underlying reserve cryptocurrency compared to when they minted their ReserveCoins (the increased amount coming from users who minted AgeUSD).

As such Reserve Providers allow AgeUSD users to enjoy stability of value. On their end, the Reserve Providers absorb the potential upside (if the value of the reserves goes up via the price of Ergs increasing compared to USD) but also absorb the potential downside (if the underlying cryptocurrency in the reserve goes down in price).

This provides individuals with the ability to choose to either go "long" Ergs (via minting ReserveCoins), or to choose stability (via minting AgeUSD).

For more detailed explanations check out:

1. [A Pegged and Crypto-Backed Algorithmic Stablecoin - Bruno Woltzenlogel Paleo - Ergo Summit 2021](https://github.com/Emurgo/age-usd/blob/main/docs/A%20Pegged%20and%20Crypto-Backed%20Algorithmic%20Stablecoin%20Slides%20-%20Bruno%20Woltzenlogel%20Paleo%20-%20Ergo%20Summit%202021.pdf)
2. [AgeUSD Smart Contract Protocol Overview - Amitabh Saxena - Ergo Summit 2021](https://github.com/Emurgo/age-usd/blob/main/docs/AgeUSD%20Smart%20Contract%20Protocol%20Overview%20Slides%20-%20Amitabh%20Saxena%20-%20Ergo%20Summit%202021.pdf)
3. The [AgeUSD stories](docs/stories.md) short summary document.

## Fees

The AgeUSD protocol fee parameter defaults are:

1. 1% Protocol Fee
2. 0.25% Frontend Implementor Fee

The protocol fee is charged on all minting/redeeming actions in the protocol (for both AgeUSD & ReserveCoins). The Ergs from this fee go back to the protocol reserves, and as such profit the ReserveCoin holders directly. In other words, if you hold ReserveCoins, you are not only rewarded in the scenario that the price of Erg goes up, but also if a lot of people use the AgeUSD protocol. This provides further incentives for Reserve Providers to ensure sufficient liquidity is always available so AgeUSD users can always mint AgeUSD.

The frontend implementor fee is the fee that gets paid out to the frontend implementor who built a GUI on top of the AgeUSD headless dApp. This fee payout is performed automatically as a part of every mint/redeem action, and the frontend implementor simply needs to provide their address as an input to the action method, thereby incentivizing a future ecosystem of decentralized & competing AgeUSD frontends.

These fees are adjustable by the deployer of the AgeUSD protocol on-chain, and as such are simply recommended defaults.

## Traversing This Repository

This repo holds the specifications, smart contracts, and off-chain code(headless dApp) for the AgeUSD protocol.

### AgeUSD Specs

[The AgeUSD Specs](ageusd-specs) are the high-level specifications of the AgeUSD protocol in the EUTXO model on top of Ergo. These specs start from `v0.1` showing off the addition of various features over the course of the design process of the AgeUSD protocol.

### AgeUSD Smart Contracts

[The AgeUSD Smart Contracts](ageusd-smart-contracts) are written in ErgoScript and also start from `v0.1` and increase upwards as new features were added or bugs were fixed.

### AgeUSD Headless dApp

[The AgeUSD Headless dApp](ageusd-headless) is the off-chain code which provides developers with a pure and portable interface for both reading the current state of a deployed instance of AgeUSD on-chain, performing Actions in the protocol, as well as containing a few helper methods to make the lives of front-end implementors easier.

Readable AgeUSD State:

- Base Reserves
- Liabilities
- Equity
- Number Of Circulating StableCoins
- Number Of Circulating ReserveCoins
- StableCoin Nominal Price
- ReserveCoin Nominal Price

Actions:

- Mint StableCoin
- Mint ReserveCoin
- Redeem StableCoin
- Redeem ReserveCoin

Helper Methods:

- Cost To Mint X StableCoins
- Cost To Mint Y ReserveCoins
- Amount Received From Redeeming X StableCoins
- Amount Received From Redeeming Y ReserveCoins

### AgeUSD CLI

[The AgeUSD CLI](ageusd-cli) uses the AgeUSD Headless dApp and implements a command line interface for interacting with a deployment of the protocol on-chain. The CLI use an Ergo Node for UTXO-set scanning to find the required boxes + posting the transactions.

The CLI is primarily geared to be used by technical users to interact with a deployment of AgeUSD, and to be an example for frontend developers to understand how to implement a frontend when looking to create a GUI. (Do note, the AgeUSD Headless dApp provides an interface for finding all input UTXOs without using UTXO-set scans as well, making the developer experience very streamlined)

## Related Works

The AgeUSD protocol was inspired by the [staticoin protocol](http://staticoin.com/whitepaper.pdf), however redesigned from the ground-up to fit the EUTXO model while providing several improvements to drastically improve the stability of the stablecoin. As such we reap the benefits of avoiding CDPs, while overcoming some of the pitfalls that the staticoin protocol ran into (lack of mechanisms to overcompensate reserves to manage volatility being the largest).
