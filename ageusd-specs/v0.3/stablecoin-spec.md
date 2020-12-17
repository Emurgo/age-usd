# Updatable AgeUSD Protocol Informal Specification v0.3

This is an informal specification of the AgeUSD protocol v0.3, which is an enhancement to the [v0.2 specification](https://github.com/Emurgo/stablecoin-project/blob/utxo-spec/utxo-spec/v0.2/stablecoin-spec.md).

This specification follows [Ergo Improvement Proposal 6: Informal Smart Contract Protocol Specification Format](https://github.com/ergoplatform/eips/blob/master/eip-0006.md).

## Differences with v0.2
- Fee in base currency (Ergs) to every exchange transaction. The fee is a fixed fraction of the absolute value of the base currency exchanged.
- Cooling-off period (defined at bootstrap, usually of 2 weeks), during which `maxReserveRatio` is infinite.
- The StableCoin is pegged to USD-cent instead of USD. That is, one StableCoin = one USD-cent. Note that the rate provided by the current oracle-pool is in `nano-ERGs per USD`, and will be converted accordingly.

## Terminology

1. The *Peg Currency*, **P**, (example USD, USD-cent or Euro) is the currency that the StableCoin will be pegged to (attempting to approach 1:1).
2. The *Base Currency*, **B** (example nano-ERG) is the cryptocurrency used to provide stability for the peg.
3. The StableCoin tokens will be represented by **SC**.
At any time, one SC token must be exchangeable both ways with B at or close to the rate *r* in units of B/P.
4. We also have another token called the ReserveCoin represented by **RC**. This token acts like a "shock absorber" for fluctuations in the value of *r*, while
allowing holders to profit by taking on an appropriate increase in risk.
5. We will need a reliable **Data Source** supplying the exchange rate in units of B per unit of P (e.g., nano-ERG per USD-cent).
6. We will have an **Update** NFT that uniquely identifies the box containing the update logic.

## Data Source

This can be implemented using an *Oracle* or an *Oracle Pool*.
We abstract away such details and assume that the data source, given as a data input, is identified by a *data-source NFT* (DS) and contains
the rate in its register `R4`.


## Stage ToC
1. [Bank](<#Stage-Bank>)
2. [Receipt](<#Stage-Receipt>)
3. [Update](<#Stage-Update>)
4. [Ballot](<#Stage-Ballot>)


## Action ToC
1. [Bootstrap](<#Action-Bootstrap>)
2. [Exchange StableCoins](<#Action-Exchange-StableCoins>)
3. [Exchange ReserveCoins](<#Action-Exchange-ReserveCoins>)
4. [Vote For Update](<#Action-Vote-for-Update>)
5. [Finalize Vote](<#Action-Finalize-Vote>)
6. [Update Protocol](<#Action-Update-Protocol>)

---

## Stage: Bank
This box defines the main stage of the protocol and stores the following:

1. Unminted StableCoins tokens
2. Unminted ReserveCoins tokens
3. Base reserves (NanoErgs)

Furthermore, the box also holds a singleton token (NFT/Bank Token) which makes it unique. This prevents bad actors from tricking users into using the wrong dApp and potentially stealing their funds (it also makes the box easier to find via UTXO-set scanning).

The maximum supply of both StableCoins and ReserveCoins is hard-coded into the smart contract. Additionally, the current supply of both coins is stored in registers R4 and R5.

#### Hard-coded Values
- Data Source NFT id that authenticates the data source. This will be the oracle pool NFT ID.
- Minimum reserve ratio permitted as a percent (`minReserveRatioPercent`). This should usually be higher than 100 (%). A typical value is 400 (%)
- Maximum reserve ratio permitted as a percent (`maxReserveRatioPercent`). This should be much higher than `minReserveRatioPercent`. A typical value is 800 (%)
- Minimum storage rent needed (`minStorageRent`), the minimum number of Ergs to keep in box at all times.
- Update NFT id that authenticates the update stage box.
- Fee to be applied to exchange in percent.

#### Tokens
1. StableCoin Tokens at index 0
2. ReserveCoin Tokens at index 1
3. Bank NFT uniquely identifying the UTXO at index 2

#### Registers
1. R4. The current number of StableCoins in circulation (Long)
2. R5. The current number of reserveCoins in circulation (Long)

#### Mandatory Stage Spending Conditions
- This [Bank](<#Stage-Bank>) box is the first input.
- The first data Input contains the Data Source NFT at the first token index.
- The first output contains the same script as this box (the [Bank](<#Stage-Bank>) stage UTXO) with the following additional conditions:
    * The first three token IDs are the same as the token IDs of this box, and in the same order.
    * The quantity of the first three tokens must be 1 or more.

#### Actions/Spending Paths
- [Exchange StableCoins](<#Action-Exchange-StableCoins>)
- [Exchange ReserveCoins](<#Action-Exchange-ReserveCoins>)
- [Update Protocol](<#Action-Update-Protocol>)



## Stage: Receipt
A box at this stage is owned by a user of the protocol after they have performed any of [Exchange-StableCoins](<#Action-Exchange-StableCoins>) or
[Exchange-ReserveCoins](<#Action-Exchange-ReserveCoins>) actions.
This stage holds the receipt of the exchange.

In particular, its register R4 will contain the amount of StableCoins or ReserveCoins ergs transferred
as a result of either a mint or redeem operation.

A sufficient minimal number of ergs must be held within this box in order to exist on the blockchain.

The smart contracts do not enforce any Ergs withdrawn from the Bank to be stored in this box. However, for consistency, the off-chain component **should** store
the Ergs in this box.

#### Registers

1. R4. Delta in circulating StableCoins or ReserveCoins (Long). This is the (non-zero) amount redeemed or minted of StableCoin. This can be negative (redeem) or positive (mint).
2. R5. Delta in nanoErg reserves (Long). This is the amount of Ergs added to bank. If negative then nanoErgs are removed from the bank.

#### Tokens

The first index can contain a non-zero quantity of either StableCoin or ReserveCoin tokens if necessary, otherwise it will be empty.
Note: for efficiency, the smart contract will not enforce token or Ergs to be stored in Receipt box.
However, for consistency the off-chain component **should** store both in this box.



## Stage: Update
A box at this stage contains the rules for updating the Bank box contract. We allow updating only the Bank box's address, preserving the remaining values (apart from boxId).
In particular, the register R4 of this box will contain the hash of the ErgoTree of the new Bank stage (the ErgoTree uniquely determines the address).
This value is determined via votes using the [Collect-Ballots](<#Action-Finalize-Vote>) Action.

#### Registers

1. R4. Hash of the ErgoTree of the new bank box.

#### Hard-coded Values
- Bank box NFT that authenticates the Bank box.
- Ballot token Id that autneiticates Ballot boxes.
- Min number of ballots needed for a successful vote.

#### Tokens
1. Update NFT at index 0

#### Actions/Spending Paths
- [Collect-Ballots](<#Action-Finalize-Vote>)
- [Update Protocol](<#Action-Update-Protocol>)


## Stage: Ballot
A box at this stage holds one or more votes for the new Bank contract address. If sufficient number of Ballot boxes
 are collected such that the total number of votes reach the minimum needed then the Update stage box can be spent via the
 [Collect-Ballots](<#Action-Finalize-Vote>) action.
There is no restriction on the script of this box. The only requirement is that the box should be spendable only by someone who
should have the right to vote for updating the Bank box.

#### Registers

1. R4. Hash of the ErgoTree of the new bank box for which this vote is cast.
2. R5. Box Id of the current [Update stage](<#Stage-Update>) box.

#### Tokens
1. One or more ballot tokens at index 0

#### Actions/Spending Paths
- [Cast-Ballot](<#Action-Vote-for-Update>)

---


## Action: Bootstrap
Instantiating the protocol requires issuing this [Bootstrap](<#Action-Bootstrap>) action.

Prior to bootstrap, three tokens must be created:
1. A StableCoin token in quantity [maximum number of allowed StableCoins tokens].
2. A ReserveCoin token in quantity [maximum number of allowed ReserveCoins tokens].
3. A NFT/Singleton token (in quantity 1) that will be used to identify the StableCoin dApp instance.

#### Inputs
1. A box with maximum number of allowed StableCoins tokens.
2. A box with maximum number of allowed ReserveCoins tokens.
3. A box with the generated NFT/Singleton token.

Note that the maximum number of allowed tokens for both StableCoins and ReserveCoins must be 1 more than the maximum number of allowed coins in circulation.
This is because the contract requires the Bank output box to contain at least one token of each type.

#### Outputs

1. A box in the [Bank](<#Stage-Bank>) stage with all of the tokens

---

## Action: Exchange StableCoins
This action allows a user to do one of the following sub-actions:

1. Mint StableCoins
2. Redeem StableCoins

Both cases are handled by the `Exchange Formula`, which validates that the mint/redeem amounts are valid.
The off-chain portion of the dApp will implement each of these sub-actions as distinct functions,
however on-chain they are merely validated via the `Exchange Formula`.

#### Rules
At a high level, the rules of the exchange are as follows.

##### Mint StableCoins

1. Let `scCircDelta` (to be stored in the second output's R4) be the amount of StableCoins minted. In this case `scCircDelta` will be positive.
2. The number of StableCoins in circulation (stored in R4) must be increased by `scCircDelta`.
3. The number of un-minted StableCoins (stored in tokens(0) of the Bank box) must be decremented by `scCircDelta`.
4. The number of nanoErgs must be incremented by exactly `bcReserveDeltaExpected` given by the formula below.
5. The released tokens can be stored anywhere (but the recommendation is to store it in the Receipt stage box).

The rules for Redeem are similar with the signs reversed.
The rules for ReserveCoins are also similar, the difference being in the exchange rate.

#### Formulas

##### Token IDs
```scala
val stableCoinTokenId = SELF.tokens(0)._1
val oraclePoolNFT = CONTEXT.dataInputs(0).tokens(0)._1
```

##### Exchange Rate (units of B per units of P)
```scala
val rate = CONTEXT.dataInputs(0).R4[Long].get / 100 // nano-ERGs per USD-cent
```

##### Conservation Rules
```scala
val inBox = SELF
val outBox = OUTPUTS(0)
val receiptBox = OUTPUTS(1)

val bcReserveIn = inBox.value - minStorageRent
val bcReserveOut = outBox.value - minStorageRent

val scTokensIn = inBox.tokens(0)._2
val scTokensOut = outBox.tokens(0)._2

val scCircDelta = receiptBox.R4[Long].get
val bcReserveDelta = receiptBox.R5[Long].get

val scCircIn = inBox.R4[Long].get
val scCircOut = outBox.R4[Long].get

require(scCircDelta != 0)
require(scCircOut == scCircIn + scCircDelta)
require(scTokensOut == scTokensIn - scCircDelta)
require(bcReserveOut >= bcReserveIn + bcReserveDelta)
require(scCircOut >= 0)
require(bcReserveOut >= 0)

// token ordering for all tokens and quantities for remaining tokens must be preserved in newBox (not shown here)
```

##### Exchange Rules
```scala
val bcReserveNeededIn = scCircIn * rate
val liabilities = bcReserveIn.min(bcReserveNeededIn)

val liableRate = if (scCircIn == 0) INF else liabilities / scCircIn
val scNominalPrice = rate.min(liableRate)
val bcReserveDeltaExpected = scNominalPrice * scCircDelta

val signedFee = bcReserveDeltaExpected * feePercent / 100
val actualFee = if (signedFee < 0) -signedFee else signedFee
val bcReserveDeltaExpectedWithFee = bcReserveDeltaExpected + actualFee

val bcReserveNeededOut = scCircOut * rate
val reserveRatioPercentOut = if (bcReserveNeededOut == 0)
                                maxReserveRatioPercent
                             else
                                bcReserveOut * 100 / bcReserveNeededOut

require(bcReserveDelta == bcReserveDeltaExpectedWithFee)
if (scCircDelta > 0) { // minting
  require(reserveRatioPercentOut >= minReserveRatioPercent)
  // minting SC should be allowed only if final reserve ratio is above min ratio
} // redeeming SC should be allowed in all circumstances
```

#### Data-Inputs
1. Oracle Pool box which holds an NFT corresponding to the *data-source NFT*.

#### Inputs
1. The [Bank](<#Stage-Bank>) box.
2. Input boxes which may hold Ergs, stablecoins, or reservecoins. (Depending on the exchange sub-action taking place)

#### Outputs

1. A new [Bank](<#Stage-Bank>) box.
2. A [Receipt](<#Stage-Receipt>) box which contains the result of the transaction.

#### Action Conditions
- The first output contains tokens with the following conditions:
    * The Ergs and the quantity of the first two tokens as per the deltas stored in the second output.
- The second output contains a box at the [Receipt](<#Stage-Receipt>) stage. The registers of that box indicate the deltas of the following:
    * R4 contains `scCircDelta`, the delta in circulating StableCoins (should not be 0)
    * R5 contains `bcReserveDelta`, the delta in locked base reserves (in base currency, nanoErgs)
- The values and registers of output bank stage box are as per the deltas above applied
- The value `scCircDelta` is used to calculate the expected delta in base currency `bcReserveDeltaExpected`. Then
  `bcReserveDelta == bcReserveDeltaExpected`.

## Action: Exchange ReserveCoins
This action allows a user to do one of the following sub-actions:

1. Mint ReserveCoins
2. Redeem ReserveCoins

All of these cases are handled by the `Exchange Formula`, which validates that the mint/redeem amounts are valid.
The off-chain portion of the dApp will implement each of these sub-actions as distinct functions,
however on-chain they are merely validated via the `Exchange Formula`.

#### Rules
The rules of the exchange are as follows:

##### Mint ReserveCoins

1. Let `rcCircDelta` be the amount of ReserveCoins minted. In this case `rcCircDelta` will be positive.
2. The number of ReserveCoins in circulation (in R5 of the Bank box) must be increased by `rcCircDelta`.
3. The number of unminted ReserveCoins (in tokens(1) of the bank box) must be decremented by `rcCircDelta`.
4. The number of nanoErgs must be incremented by exactly `bcReserveDeltaExpected` given by the formula below.
5. The released tokens can be stored anywhere (but the recommendation is to store it in the Receipt stage box).

The rules for Redeem are similar with the signs reversed.

#### Formulas

##### Token IDs
```scala
val reserveCoinTokenId = SELF.tokens(1)._1
val oraclePoolNFT = CONTEXT.dataInputs(0).tokens(0)._1
```

##### Exchange Rate (units of B per units of P)
```scala
val rate = CONTEXT.dataInputs(0).R4[Long].get / 100 // nano-ERGs per USD-cent
```
##### Conservation Rules
```scala
val inBox = SELF
val outBox = OUTPUTS(0)
val receiptBox = OUTPUTS(1)

val bcReserveIn = inBox.value - minStorageRent
val bcReserveOut = outBox.value - minStorageRent

val rcTokensIn = inBox.tokens(1)._2
val rcTokensOut = outBox.tokens(1)._2

val rcCircDelta = receiptBox.R4[Long].get
val bcReserveDelta = receiptBox.R5[Long].get

val scCircIn = inBox.R4[Long].get
val rcCircIn = inBox.R5[Long].get
val rcCircOut = outBox.R5[Long].get

require(rcCircDelta != 0)
require(rcCircOut = rcCircIn + rcCircDelta)
require(rcTokensOut == rcTokensIn - rcCircDelta)
require(bcReserveOut >= bcReserveIn + bcReserveDelta)
require(rcCircOut >= 0)
require(bcReserveOut >= 0)

// token ordering for all tokens and quantities for remaining tokens must be preserved in newBox (not shown here)

```

##### Exchange Rules
```scala
val bcReserveNeededIn = scCircIn * rate
val liabilities = bcReserveIn.min(bcReserveNeededIn)

val equity = bcReserveIn - liabilities
val rcRate = if (rcCircIn == 0) rcDefaultPrice else equity / rcCircIn
val rcNominalPrice = if (equity == 0) rcMinPrice else rcRate
val bcReserveDeltaExpected = rcNominalPrice * rcCircDelta

val signedFee = bcReserveDeltaExpected * feePercent / 100
val actualFee = if (signedFee < 0) -signedFee else signedFee
val bcReserveDeltaExpectedWithFee = bcReserveDeltaExpected + actualFee


val bcReserveNeededOut = bcReserveNeededIn // scCircIn == scCircOut because this is rc exchange

val reserveRatioPercentOut = if (bcReserveNeededOut == 0)
                                maxReserveRatioPercent
                             else
                                bcReserveOut * 100 / bcReserveNeededOut

val reserveRatioPercentIn = if (bcReserveNeededOut == 0)
                               maxReserveRatioPercent
                            else
                               bcReserveIn * 100 / bcReserveNeededOut

require(bcReserveDelta == bcReserveDeltaExpectedWithFee)
if (rcCircDelta > 0) { // minting
  require(reserveRatioPercentOut <= maxReserveRatioPercent)
  // minting RC should be allowed as long as (1) out-reserve-ratio is below max and (2) out-reserve-ratio > in-reserve-ratio
} else { // redeeming
  require(reserveRatioPercentOut >= minReserveRatioPercent)
  // redeeming RC should be allowed only if final reserve ratio is within limits
}
```

#### Data-Inputs
1. Oracle Pool box which holds an NFT corresponding to the *data-source NFT*.

#### Inputs
1. The [Bank](<#Stage-Bank>) box.
2. Input boxes which may hold Ergs and/or ReserveCoins. (Depending on the exchange sub-action taking place)

#### Outputs
1. A new [Bank](<#Stage-Bank>) box.
2. A [Receipt](<#Stage-Receipt>) box which contains the result of the transaction.

#### Action Conditions
- The first output contains tokens with the following conditions:
    * The Ergs and the quantity of the first two tokens as per the deltas stored in the second output.
- The second output contains a box at the [Receipt](<#Stage-Receipt>) stage. The registers of that box indicate the deltas of the following:
    * R4 contains `rcCircDelta`, the delta in circulating ReserveCoins (should not be 0)
    * R5 contains `bcReserveDelta`, the delta in locked base reserves (in base currency, nanoErgs)
- The values and registers of output bank stage box are as per the deltas above applied
- The value `scCircDelta` is used to calculate the expected delta in base currency `bcReserveDeltaExpected`. Then
  `bcReserveDelta == bcReserveDeltaExpected`.



## Action: Vote For Update
Casting a ballot involves committing to the new address of the Bank stage box.
Prior to this, the ErgoTree of the new Bank box must be known. Let `newBankScriptHash` be the hash of this value.
Additionally. the box Id of the current Update box stage must be known. Let this be `updateBoxId`.

#### Inputs
1. The [Ballot](<#Stage-Ballot>) box with one or more ballot tokens.

#### Outputs
1. The new [Ballot](<#Stage-Ballot>) box with one or more ballot tokens such that its R4 contains `newBankScriptHash` and R5 contains `updateBoxId`.



## Action: Finalize Vote

Collecting ballots involves collecting all valid votes such that the minimum voting threshold is reached.

#### Data Inputs
1. Several [Ballot](<#Stage-Ballot>) boxes with one or more ballot tokens each.
The R4 of all Ballot boxes must be identical. Let us call this value `newBankScriptHash`.
Additionally, R5 of all Ballot boxes must be identical. Let us call this value `updateBoxId`

#### Inputs
1. The Current [Update](<#Stage-Update>) box.

#### Outputs
1. The new [Update](<#Stage-Update>).

#### Action Conditions
1. The box Id of the input Update box must be `updateBoxId`.
2. There should be at least `minVotes` number of total ballot tokens in the data inputs.
3. R4 of the output Update box must contain `newBankScriptHash`.



## Action: Update Protocol

Updating bank involves spending the Update and Bank stage boxes and generating the same update box and the new bank box.

#### Inputs
1. The [Update](<#Stage-Update>) box.
2. The [Bank](<#Stage-Bank>) box.

#### Outputs
1. The [Update](<#Stage-Update>) box with everything preserved.
2. The [Bank](<#Stage-Bank>) box with updated address.

#### Action Conditions
1. R4 of the input Update box must equal the hash of the script of the output Bank box.
2. The addresses of the input and output Bank boxes must be different
3. The values, registers and tokens of the input Bank and Update boxes must be preserved in the respective output boxes.
4. The input Update box has the update NFT.
5. The input Bank box has the bank NFT.

