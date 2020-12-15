The AgeUSD Stablecoin Protocol
-------------------------

AgeUSD is a novel crypto-backed stablecoin protocol that has been created in joint partnership by the [Ergo Foundation](https://ergoplatform.org/en/foundation/), [Emurgo](https://emurgo.io/), and [IOG](https://iohk.io/en/about/) on top of the [Ergo Blockchain](https://ergoplatform.org/).


...

## How Does The AgeUSD Protocol Work?




## Traversing This Repository
This repo holds the specifications, smart contracts, and off-chain code(headless dApp) for the AgeUSD protocol.


### AgeUSD Headless dApp
The AgeUSD Headless dApp provides developers with a pure and portable interface for both reading the current state of AgeUSD on-chain, performing Actions in the dApp, as well as a few helper methods to make the lives of front-end implementors easier.

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