use ergo_headless_dapp_framework::ErgUsdOraclePoolBox;
use ergo_headless_dapp_framework::NanoErg;

/// Calculates the Reserve Ratio based on provided inputs.
pub fn reserve_ratio(
    base_reserves: NanoErg,
    circulating_stablecoins: u64,
    oracle_rate: NanoErg,
) -> u64 {
    if base_reserves == 0 || oracle_rate == 0 || circulating_stablecoins == 0 {
        return 0;
    }
    (base_reserves * 100) / (circulating_stablecoins * oracle_rate)
}
