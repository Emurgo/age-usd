package stablecoin.v2

private[v2] object Contracts {
  val oraclePoolNFT = "08b59b14e4fdd60e5952314adbaa8b4e00bc0f0b676872a5224d3bf8591074cd"

  val updateNFT = "" // quantity 1
  val ballotTokenId = "" // quantity 1000

  val bankNFT = "" // quantity 1
  val scToken = "" // quantity 10000000000000
  val rcToken = "" // quantity 10000000000000

  val minReserveRatioPercent = 400 // percent
  val maxReserveRatioPercent = 800 // percent

  val rcDefaultPrice = 1000000

  val minStorageRent = 10000000L
  val longMax = Long.MaxValue
  val bankScript =
    s"""{ // this box
       |  // R4: Number of stable-coins in circulation
       |  // R5: Number of reserve-coins in circulation
       |
       |  val isExchange = if (CONTEXT.dataInputs.size > 0) {
       |
       |    val dataInput = CONTEXT.dataInputs(0)
       |    val validDataInput = dataInput.tokens(0)._1 == oraclePoolNFT
       |
       |    val bankBoxIn = SELF
       |    val bankBoxOut = OUTPUTS(0)
       |
       |    val rateBox = dataInput
       |    val receiptBox = OUTPUTS(1)
       |
       |    val rate = rateBox.R4[Long].get
       |
       |    val scCircIn = bankBoxIn.R4[Long].get
       |    val rcCircIn = bankBoxIn.R5[Long].get
       |    val bcReserveIn = bankBoxIn.value - $minStorageRent
       |
       |    val scTokensIn = bankBoxIn.tokens(0)._2
       |    val rcTokensIn = bankBoxIn.tokens(1)._2
       |
       |    val scCircOut = bankBoxOut.R4[Long].get
       |    val rcCircOut = bankBoxOut.R5[Long].get
       |    val bcReserveOut = bankBoxOut.value - $minStorageRent
       |
       |    val scTokensOut = bankBoxOut.tokens(0)._2
       |    val rcTokensOut = bankBoxOut.tokens(1)._2
       |
       |    val totalScIn = scTokensIn + scCircIn
       |    val totalScOut = scTokensOut + scCircOut
       |
       |    val totalRcIn = rcTokensIn + rcCircIn
       |    val totalRcOut = rcTokensOut + rcCircOut
       |
       |    val rcExchange = rcTokensIn != rcTokensOut
       |    val scExchange = scTokensIn != scTokensOut
       |
       |    val rcExchangeXorScExchange = (rcExchange || scExchange) && !(rcExchange && scExchange)
       |
       |    val circDelta = receiptBox.R4[Long].get
       |    val bcReserveDelta = receiptBox.R5[Long].get
       |
       |    val rcCircDelta = if (rcExchange) circDelta else 0L
       |    val scCircDelta = if (rcExchange) 0L else circDelta
       |
       |    val validDeltas = (scCircIn + scCircDelta == scCircOut) &&
       |                      (rcCircIn + rcCircDelta == rcCircOut) &&
       |                      (bcReserveIn + bcReserveDelta == bcReserveOut)
       |
       |    val coinsConserved = totalRcIn == totalRcOut && totalScIn == totalScOut
       |
       |    val tokenIdsConserved = bankBoxOut.tokens(0)._1 == bankBoxIn.tokens(0)._1 && // also ensures that at least one token exists
       |                            bankBoxOut.tokens(1)._1 == bankBoxIn.tokens(1)._1 && // also ensures that at least one token exists
       |                            bankBoxOut.tokens(2)._1 == bankBoxIn.tokens(2)._1    // also ensures that at least one token exists
       |
       |    val mandatoryRateConditions = rateBox.tokens(0)._1 == oraclePoolNFT
       |    val mandatoryBankConditions = bankBoxOut.value >= $minStorageRent &&
       |                                  rcExchangeXorScExchange &&
       |                                  coinsConserved &&
       |                                  validDeltas &&
       |                                  tokenIdsConserved
       |
       |    // exchange equations
       |    val bcReserveNeededOut = scCircOut * rate
       |    val bcReserveNeededIn = scCircIn * rate
       |    val liabilitiesIn = min(bcReserveIn, bcReserveNeededIn)
       |    val reserveRatioPercentOut = if (bcReserveNeededOut == 0) maxReserveRatioPercent else bcReserveOut * 100 / bcReserveNeededOut
       |
       |    val validReserveRatio = if (scExchange) {
       |      if (scCircDelta > 0) {
       |        reserveRatioPercentOut >= minReserveRatioPercent
       |      } else true
       |    } else {
       |      if (rcCircDelta > 0) {
       |        reserveRatioPercentOut <= maxReserveRatioPercent
       |      } else {
       |        reserveRatioPercentOut >= minReserveRatioPercent
       |      }
       |    }
       |
       |    val brDeltaExpected = if (scExchange) { // sc
       |      val liableRate = if (scCircIn == 0) ${longMax}L else liabilitiesIn / scCircIn
       |      val scNominalPrice = min(rate, liableRate)
       |      scNominalPrice * scCircDelta
       |    } else { // rc
       |      val equityIn = bcReserveIn - liabilitiesIn
       |      val equityRate = if (rcCircIn == 0) ${rcDefaultPrice}L else equityIn / rcCircIn
       |      val rcNominalPrice = if (equityIn == 0) ${rcDefaultPrice}L else equityRate
       |      rcNominalPrice * rcCircDelta
       |    }
       |
       |    mandatoryRateConditions &&
       |    mandatoryBankConditions &&
       |    bcReserveDelta == brDeltaExpected &&
       |    validReserveRatio &&
       |    validDataInput 
       |  } else false
       |
       |  sigmaProp(isExchange || INPUTS(0).tokens(0)._1 == updateNFT)
       |}
       |""".stripMargin

  val minVotes: Int = 5

  val updateScript =
    s"""{ // This box:
       |  // R4 the "control value" (such as the hash of a script of some other box)
       |  // 
       |  // ballot boxes (data Inputs)
       |  // R4 the new control value
       |  // R5 the box id of this box
       |  
       |  val updateBoxIn = INPUTS(0)
       |  val updateBoxOut = OUTPUTS(0)
       |  val validIn = SELF.id == INPUTS(0).id 
       |    
       |  val voteSuccessPath = {
       |    val newValue = updateBoxOut.R4[Coll[Byte]].get
       |    val oldValue = updateBoxIn.R4[Coll[Byte]].get
       |
       |    val validOut = updateBoxOut.propositionBytes == updateBoxIn.propositionBytes && 
       |                   updateBoxOut.value >= $minStorageRent &&
       |                   updateBoxOut.tokens == updateBoxIn.tokens &&
       |                   newValue != oldValue 
       |
       |    def validBallotSubmissionBox(b:Box) = b.tokens(0)._1 == ballotTokenId && 
       |                                          b.R4[Coll[Byte]].get == newValue && // ensure that vote is for the newValue
       |                                          b.R5[Coll[Byte]].get == SELF.id  // ensure that vote counts only once 
       |  
       |    val ballots = CONTEXT.dataInputs.filter(validBallotSubmissionBox)
       |
       |    val ballotCount = ballots.fold(0L, { (accum: Long, box: Box) => accum + box.tokens(0)._2 })
       |  
       |    val voteAccepted = ballotCount >= $minVotes
       |  
       |    validOut && voteAccepted
       |  } 
       |  
       |  val updatePath = {
       |    val bankBoxIn = INPUTS(1)
       |    val bankBoxOut = OUTPUTS(1)
       |    
       |    val storedNewHash = SELF.R4[Coll[Byte]].get 
       |    val bankBoxOutHash = blake2b256(bankBoxOut.propositionBytes)
       |    
       |    val validBankBox = bankBoxIn.tokens(2)._1 == bankNFT && // bank box is first input
       |                       bankBoxIn.tokens == bankBoxOut.tokens &&
       |                       storedNewHash == bankBoxOutHash && 
       |                       bankBoxIn.propositionBytes != bankBoxOut.propositionBytes &&
       |                       bankBoxIn.R4[Long].get == bankBoxOut.R4[Long].get &&
       |                       bankBoxIn.R5[Long].get == bankBoxOut.R5[Long].get && 
       |                       bankBoxIn.value == bankBoxOut.value 
       |    
       |    val validUpdateBox = updateBoxIn.R4[Coll[Byte]].get == updateBoxOut.R4[Coll[Byte]].get && 
       |                         updateBoxIn.propositionBytes == updateBoxOut.propositionBytes && 
       |                         updateBoxIn.tokens == updateBoxOut.tokens && 
       |                         updateBoxIn.value == updateBoxOut.value 
       |                          
       |    validBankBox && 
       |    validUpdateBox 
       |  } 
       |  
       |  sigmaProp(
       |    validIn && (
       |      voteSuccessPath || 
       |      updatePath
       |    )
       |  )
       |}
       |""".stripMargin
}
