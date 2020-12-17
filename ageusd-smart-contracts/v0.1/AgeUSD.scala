package stablecoin.v1

private[v1] object BankContract {
  val oraclePoolNFT = "b662db51cf2dc39f110a021c2a31c74f0a1a18ffffbf73e8a051a7b8c0f09ebc"
  
  val minStorageRent = 10000000L
  
  val minReserveRatioPercent: Long = 400
  val maxReserveRatioPercent: Long = 800

  val rcDefaultPrice = 1000000L


  val script =
    s"""{ // this box
       |  // R4: Number of stable-coins in circulation
       |  // R5: Number of reserve-coins in circulation
       |
       |  val bankBoxIn = SELF
       |  val bankBoxOut = OUTPUTS(0)
       |  val rateBox = CONTEXT.dataInputs(0)
       |  val receiptBox = OUTPUTS(1)
       |
       |  val rate = rateBox.R4[Long].get
       |
       |  val scCircIn = bankBoxIn.R4[Long].get
       |  val rcCircIn = bankBoxIn.R5[Long].get
       |  val bcReserveIn = bankBoxIn.value - $minStorageRent
       |
       |  val scTokensIn = bankBoxIn.tokens(0)._2
       |  val rcTokensIn = bankBoxIn.tokens(1)._2
       |
       |  val scCircOut = bankBoxOut.R4[Long].get
       |  val rcCircOut = bankBoxOut.R5[Long].get
       |  val bcReserveOut = bankBoxOut.value - $minStorageRent
       |
       |  val scTokensOut = bankBoxOut.tokens(0)._2
       |  val rcTokensOut = bankBoxOut.tokens(1)._2
       |
       |  val totalScIn = scTokensIn + scCircIn
       |  val totalScOut = scTokensOut + scCircOut
       |
       |  val totalRcIn = rcTokensIn + rcCircIn
       |  val totalRcOut = rcTokensOut + rcCircOut
       |
       |  val rcExchange = rcTokensIn != rcTokensOut
       |  val scExchange = scTokensIn != scTokensOut
       |
       |  val rcExchangeXorScExchange = (rcExchange || scExchange) && !(rcExchange && scExchange)
       |
       |  val circDelta = receiptBox.R4[Long].get
       |  val bcReserveDelta = receiptBox.R5[Long].get
       |
       |  val rcCircDelta = if (rcExchange) circDelta else 0L
       |  val scCircDelta = if (rcExchange) 0L else circDelta
       |
       |  val validDeltas = (scCircIn + scCircDelta == scCircOut) &&
       |                    (rcCircIn + rcCircDelta == rcCircOut) &&
       |                    (bcReserveIn + bcReserveDelta == bcReserveOut)
       |
       |  val coinsConserved = totalRcIn == totalRcOut && totalScIn == totalScOut
       |
       |  val tokenIdsConserved = bankBoxOut.tokens(0)._1 == bankBoxIn.tokens(0)._1 && // also ensures that at least one token exists
       |                          bankBoxOut.tokens(1)._1 == bankBoxIn.tokens(1)._1 && // also ensures that at least one token exists
       |                          bankBoxOut.tokens(2)._1 == bankBoxIn.tokens(2)._1    // also ensures that at least one token exists
       |
       |  val mandatoryRateConditions = rateBox.tokens(0)._1 == oraclePoolNFT
       |  val mandatoryBankConditions = bankBoxOut.value >= $minStorageRent &&
       |                                rcExchangeXorScExchange &&
       |                                coinsConserved &&
       |                                validDeltas &&
       |                                tokenIdsConserved
       |
       |  // exchange equations
       |  // common part
       |  val bcReserveNeededOut = scCircOut * rate
       |  val bcReserveNeededIn = scCircIn * rate
       |  val liabilitiesIn = min(bcReserveIn, bcReserveNeededIn)
       |  val reserveRatioPercentOut = if (bcReserveNeededOut == 0) ${maxReserveRatioPercent}L else bcReserveOut * 100 / bcReserveNeededOut
       |
       |  val validReserveRatio = if (scExchange) {
       |    if (scCircDelta > 0) {
       |      reserveRatioPercentOut >= $minReserveRatioPercent
       |    } else true
       |  } else {
       |    if (rcCircDelta > 0) {
       |      reserveRatioPercentOut <= $maxReserveRatioPercent
       |    } else {
       |      reserveRatioPercentOut >= $minReserveRatioPercent
       |    }
       |  }
       |
       |  val brDeltaExpected = if (scExchange) { // sc
       |    val liableRate = if (scCircIn == 0) ${INF}L else liabilitiesIn / scCircIn
       |    val scNominalPrice = min(rate, liableRate)
       |    scNominalPrice * scCircDelta
       |  } else { // rc
       |    val equityIn = bcReserveIn - liabilitiesIn
       |    val equityRate = if (rcCircIn == 0) ${rcDefaultPrice}L else equityIn / rcCircIn
       |    val rcNominalPrice = if (equityIn == 0) ${rcDefaultPrice}L else equityRate
       |    rcNominalPrice * rcCircDelta
       |  }
       |  
       |  val okToSpend = mandatoryRateConditions && 
       |                  mandatoryBankConditions && 
       |                  bcReserveDelta == brDeltaExpected && 
       |                  validReserveRatio
       |                  
       |  sigmaProp(okToSpend)
       |}
       |""".stripMargin

}
