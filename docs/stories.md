# AgeUSD Stories

The basic stories held within this document explain the basic mechanics of how AgeUSD works. Do note that these stories are the simplest examples possible and as such do not include fees or other pedantic features of the protocol to avoid complicating the scenario, and thereby making it easier to understand.

## Basic Usage - Underling Reserve Currency Prices Goes Up
Assume that:
1. The price of Erg = $1 USD
2. Minimum Reserve Ratio = 200%

Alice is an Ergo user with 100 Erg who wishes to have stability of value while using the blockchain. Bob is an Ergo user with 200 Erg who believes in the value of Erg (the cryptocurrency of Ergo) and wants to go long.

In order to go long, Bob becomes a Reserve Provider. He does this by depositing his 200 Erg into the StableCoin dApp and mints ReserveCoins (amount depending on existing reserve levels). These ReserveCoins are redeemable for underlying Erg reserves of the dApp at any point while the reserves are above the minimum reserve ratio, but will only profit Bob if the price of Ergs goes up.

Alice on the other hand decides to use our StableCoin dApp to obtain stability. As such she deposits her 100 Erg into the dApp and mints 100 USD StableCoins.

The total reserves at this point are equal to 300 Erg. 4 weeks pass and the price of Erg increases by 10%, to $1.10.

Alice decides that she wants to exit out of the stability due to the recent price spike. As such she redeems her 100 USD StableCoins using the dApp and withdraws 100 USD worth of Erg. At the current price of $1.10/Erg, her 100 USD StableCoins are redeemed for 90.90 Erg.

The dApp reserves now hold 209.1 Erg.

Bob decides to finalize his profit and redeems his minted ReserveCoins for the rest of the underlying Erg reserves. He receives 209.1 Erg.

As can be seen, Bob has profited 9.1 Erg by providing reserves (Erg) to the dApp through minting ReserveCoins. Because sufficient reserves were available, Alice could mint her StableCoins and maintain stability of value. In this case the stability of value meant that she redeemed 9.1 less Erg than when she minted her StableCoins.


## Basic Usage - Underling Reserve Currency Prices Goes Down
Given the same scenario where both Alice and Bob minted the same amount of StableCoins/ReserveCoins as in the prior example, let’s observe the effects of the price of Erg going down.

The total reserves at this point are equal to 300 Erg. 4 weeks pass and the price of Erg decreases by 10%, to $0.90.

Alice decides that she wants to exit out of the stability of her StableCoins. As such she redeems her 100 USD StableCoins using the dApp and withdraws 100 USD worth of Erg. At the current price of $0.90/Erg, her 100 USD StableCoins are redeemed for 111.12 Erg.

The dApp reserves now hold 188.88 Erg.

Bob now decides to exit out of his ReserveCoins for the rest of the reserves. He receives 188.88 Erg.

As can be seen, Bob has lost 11.12 Erg by providing reserves (Erg) to the dApp through minting ReserveCoins. Alice on the other hand maintained stability compared to the US dollar. As such Alice ended up redeemed 11.12 more Erg than she put in.