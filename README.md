The AgeUSD Stablecoin Protocol
-------------------------

AgeUSD is a novel crypto-backed stablecoin protocol that has been created in joint partnership by the [Ergo Foundation](https://ergoplatform.org/en/foundation/), [Emurgo](https://emurgo.io/), and [IOG](https://iohk.io/en/about/) on top of the [Ergo Blockchain](https://ergoplatform.org/).
This repository contains the specifications/smart contracts/off-chain code(headless dApp) of AgeUSD. Everything in this repo can be used to deploy your own AgeUSD instance on top of the Ergo Blockchain.

...

## How Does The AgeUSD Protocol Work?
At it's core the AgeUSD protocol is quite simple to understand. There are two kinds of parties who interact with the protocol:
1. Reserve Providers
2. AgeUSD Users

Reserve Providers submit Ergs (the native currency of Ergo) to the dApp’s reserves and by doing so mint “ReserveCoins”. Each of these ReserveCoins represent a portion of the underlying Erg reserves held in the dApp.

AgeUSD Users also submit Ergs to the dApp reserves however in their case they mint AgeUSD instead. This is only allowed by the protocol if there are sufficient reserves within the dApp (reserves are above the minimum reserve ratio). At any given moment an AgeUSD user can redeem their AgeUSD in exchange for an amount of Ergs from the reserves equal to the current exchange rate as sourced by the Erg-USD oracle pool.

Reserve Providers can only redeem their ReserveCoins for Ergs if the price of Ergs goes up and thus cover the value of all existing minted AgeUSD plus an extra margin. By redeeming their ReserveCoins, they profit as they receive more underlying reserve cryptocurrency compared to when they minted their ReserveCoins (the increased amount coming from users who minted AgeUSD).

As such Reserve Providers allow AgeUSD users to enjoy stability of value. On their end, the Reserve Providers absorb the potential upside (if the value of the reserves goes up via the price of Ergs increasing compared to USD) but also absorb the potential downside (if the underlying cryptocurrency in the reserve goes down in price).

This provides individuals with the ability to choose to either go "long" Ergs (via minting ReserveCoins), or to choose stability (via minting AgeUSD).


## Fees

- 1% Fee on all minting/redeeming actions
- 0.25% Implementor Fee which pays out the front-end implementor


## Traversing This Repository
This repo holds the specifications, smart contracts, and off-chain code(headless dApp) for the AgeUSD protocol.


### AgeUSD Headless dApp
The AgeUSD Headless dApp is the off-chain code which provides developers with a pure and portable interface for both reading the current state of a deployed instance of AgeUSD on-chain, performing Actions in the protocol, as well as containing a few helper methods to make the lives of front-end implementors easier.

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
The AgeUSD CLI uses the AgeUSD Headless dApp and implements a command line interface for interacting with the protocol on-chain. The CLI use an Ergo Node for UTXO-set scanning to find the required boxes + posting the transactions.




## Credits

...