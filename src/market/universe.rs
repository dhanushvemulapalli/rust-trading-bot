//! Global multi-asset instrument definitions and universe catalog.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetClass {
    UsEquity,
    Crypto,
    IndianEquity,
    JapaneseEquity,
    ChineseEquity,
    EuEquity,
    Future,
    Option,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub symbol: &'static str,
    pub name: &'static str,
    pub asset_class: AssetClass,
    pub exchange: &'static str,
    pub is_fractional: bool,
    pub min_lot_size: f64,
}

/// Catalog containing all 70+ symbols across global markets and asset classes.
pub const INSTRUMENT_UNIVERSE: &[Instrument] = &[
    // ── 1. Cryptocurrencies (Alpaca Crypto) ──────────────────────────────────
    Instrument { symbol: "BTC/USD", name: "Bitcoin", asset_class: AssetClass::Crypto, exchange: "FTX/Coinbase", is_fractional: true, min_lot_size: 0.0001 },
    Instrument { symbol: "ETH/USD", name: "Ethereum", asset_class: AssetClass::Crypto, exchange: "FTX/Coinbase", is_fractional: true, min_lot_size: 0.001 },
    Instrument { symbol: "SOL/USD", name: "Solana", asset_class: AssetClass::Crypto, exchange: "FTX/Coinbase", is_fractional: true, min_lot_size: 0.01 },
    Instrument { symbol: "DOGE/USD", name: "Dogecoin", asset_class: AssetClass::Crypto, exchange: "FTX/Coinbase", is_fractional: true, min_lot_size: 1.0 },
    Instrument { symbol: "AVAX/USD", name: "Avalanche", asset_class: AssetClass::Crypto, exchange: "FTX/Coinbase", is_fractional: true, min_lot_size: 0.1 },
    Instrument { symbol: "LINK/USD", name: "Chainlink", asset_class: AssetClass::Crypto, exchange: "FTX/Coinbase", is_fractional: true, min_lot_size: 0.1 },
    Instrument { symbol: "LTC/USD", name: "Litecoin", asset_class: AssetClass::Crypto, exchange: "FTX/Coinbase", is_fractional: true, min_lot_size: 0.01 },
    Instrument { symbol: "UNI/USD", name: "Uniswap", asset_class: AssetClass::Crypto, exchange: "FTX/Coinbase", is_fractional: true, min_lot_size: 0.1 },
    Instrument { symbol: "BCH/USD", name: "Bitcoin Cash", asset_class: AssetClass::Crypto, exchange: "FTX/Coinbase", is_fractional: true, min_lot_size: 0.01 },
    Instrument { symbol: "AAVE/USD", name: "Aave", asset_class: AssetClass::Crypto, exchange: "FTX/Coinbase", is_fractional: true, min_lot_size: 0.01 },

    // ── 2. Indian Equities (NSE) ─────────────────────────────────────────────
    Instrument { symbol: "RELIANCE.NS", name: "Reliance Industries", asset_class: AssetClass::IndianEquity, exchange: "NSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "TCS.NS", name: "Tata Consultancy Services", asset_class: AssetClass::IndianEquity, exchange: "NSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "HDFCBANK.NS", name: "HDFC Bank", asset_class: AssetClass::IndianEquity, exchange: "NSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "INFY.NS", name: "Infosys", asset_class: AssetClass::IndianEquity, exchange: "NSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "ICICIBANK.NS", name: "ICICI Bank", asset_class: AssetClass::IndianEquity, exchange: "NSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "HINDUNILVR.NS", name: "Hindustan Unilever", asset_class: AssetClass::IndianEquity, exchange: "NSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "ITC.NS", name: "ITC Limited", asset_class: AssetClass::IndianEquity, exchange: "NSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "SBIN.NS", name: "State Bank of India", asset_class: AssetClass::IndianEquity, exchange: "NSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "BHARTIARTL.NS", name: "Bharti Airtel", asset_class: AssetClass::IndianEquity, exchange: "NSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "LT.NS", name: "Larsen & Toubro", asset_class: AssetClass::IndianEquity, exchange: "NSE", is_fractional: false, min_lot_size: 1.0 },

    // ── 3. Japanese Equities (TSE) ───────────────────────────────────────────
    Instrument { symbol: "7203.T", name: "Toyota Motor Corp", asset_class: AssetClass::JapaneseEquity, exchange: "TSE", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "6758.T", name: "Sony Group Corp", asset_class: AssetClass::JapaneseEquity, exchange: "TSE", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "9984.T", name: "SoftBank Group", asset_class: AssetClass::JapaneseEquity, exchange: "TSE", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "6861.T", name: "Keyence Corp", asset_class: AssetClass::JapaneseEquity, exchange: "TSE", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "8035.T", name: "Tokyo Electron", asset_class: AssetClass::JapaneseEquity, exchange: "TSE", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "9432.T", name: "NTT Corp", asset_class: AssetClass::JapaneseEquity, exchange: "TSE", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "6501.T", name: "Hitachi Ltd", asset_class: AssetClass::JapaneseEquity, exchange: "TSE", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "7974.T", name: "Nintendo Co Ltd", asset_class: AssetClass::JapaneseEquity, exchange: "TSE", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "8306.T", name: "Mitsubishi UFJ Financial", asset_class: AssetClass::JapaneseEquity, exchange: "TSE", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "6098.T", name: "Recruit Holdings", asset_class: AssetClass::JapaneseEquity, exchange: "TSE", is_fractional: false, min_lot_size: 100.0 },

    // ── 4. Chinese & HK Equities (HKEX / SSE) ────────────────────────────────
    Instrument { symbol: "0700.HK", name: "Tencent Holdings", asset_class: AssetClass::ChineseEquity, exchange: "HKEX", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "9988.HK", name: "Alibaba Group", asset_class: AssetClass::ChineseEquity, exchange: "HKEX", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "3690.HK", name: "Meituan", asset_class: AssetClass::ChineseEquity, exchange: "HKEX", is_fractional: false, min_lot_size: 100.0 },
    Instrument { symbol: "1810.HK", name: "Xiaomi Corp", asset_class: AssetClass::ChineseEquity, exchange: "HKEX", is_fractional: false, min_lot_size: 200.0 },
    Instrument { symbol: "1211.HK", name: "BYD Company", asset_class: AssetClass::ChineseEquity, exchange: "HKEX", is_fractional: false, min_lot_size: 500.0 },
    Instrument { symbol: "0941.HK", name: "China Mobile", asset_class: AssetClass::ChineseEquity, exchange: "HKEX", is_fractional: false, min_lot_size: 500.0 },
    Instrument { symbol: "0939.HK", name: "China Construction Bank", asset_class: AssetClass::ChineseEquity, exchange: "HKEX", is_fractional: false, min_lot_size: 1000.0 },
    Instrument { symbol: "2318.HK", name: "Ping An Insurance", asset_class: AssetClass::ChineseEquity, exchange: "HKEX", is_fractional: false, min_lot_size: 500.0 },
    Instrument { symbol: "9618.HK", name: "JD.com", asset_class: AssetClass::ChineseEquity, exchange: "HKEX", is_fractional: false, min_lot_size: 50.0 },
    Instrument { symbol: "600519.SS", name: "Kweichow Moutai", asset_class: AssetClass::ChineseEquity, exchange: "SSE", is_fractional: false, min_lot_size: 100.0 },

    // ── 5. European Equities (Euronext / LSE / DAX) ──────────────────────────
    Instrument { symbol: "ASML.AS", name: "ASML Holding", asset_class: AssetClass::EuEquity, exchange: "Euronext Amsterdam", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "MC.PA", name: "LVMH Moet Hennessy", asset_class: AssetClass::EuEquity, exchange: "Euronext Paris", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "SAP.DE", name: "SAP SE", asset_class: AssetClass::EuEquity, exchange: "XETRA", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "SIE.DE", name: "Siemens AG", asset_class: AssetClass::EuEquity, exchange: "XETRA", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "TTE.PA", name: "TotalEnergies", asset_class: AssetClass::EuEquity, exchange: "Euronext Paris", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "SHEL.L", name: "Shell plc", asset_class: AssetClass::EuEquity, exchange: "LSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "AZN.L", name: "AstraZeneca", asset_class: AssetClass::EuEquity, exchange: "LSE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "SAN.PA", name: "Sanofi", asset_class: AssetClass::EuEquity, exchange: "Euronext Paris", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "AIR.PA", name: "Airbus SE", asset_class: AssetClass::EuEquity, exchange: "Euronext Paris", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "ALV.DE", name: "Allianz SE", asset_class: AssetClass::EuEquity, exchange: "XETRA", is_fractional: false, min_lot_size: 1.0 },

    // ── 6. Futures (CME / NYMEX / CBOT) ──────────────────────────────────────
    Instrument { symbol: "ES", name: "E-mini S&P 500", asset_class: AssetClass::Future, exchange: "CME", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "NQ", name: "E-mini Nasdaq 100", asset_class: AssetClass::Future, exchange: "CME", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "YM", name: "E-mini Dow Jones", asset_class: AssetClass::Future, exchange: "CBOT", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "RTY", name: "E-mini Russell 2000", asset_class: AssetClass::Future, exchange: "CME", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "CL", name: "Crude Oil WTI", asset_class: AssetClass::Future, exchange: "NYMEX", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "GC", name: "Gold Futures", asset_class: AssetClass::Future, exchange: "COMEX", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "SI", name: "Silver Futures", asset_class: AssetClass::Future, exchange: "COMEX", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "NG", name: "Natural Gas Futures", asset_class: AssetClass::Future, exchange: "NYMEX", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "ZB", name: "30-Year US Treasury Bond", asset_class: AssetClass::Future, exchange: "CBOT", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "ZN", name: "10-Year US Treasury Note", asset_class: AssetClass::Future, exchange: "CBOT", is_fractional: false, min_lot_size: 1.0 },

    // ── 7. Options (US Equity / Index Options) ───────────────────────────────
    Instrument { symbol: "SPY", name: "SPY Option Underlying", asset_class: AssetClass::Option, exchange: "CBOE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "QQQ", name: "QQQ Option Underlying", asset_class: AssetClass::Option, exchange: "CBOE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "AAPL", name: "AAPL Option Underlying", asset_class: AssetClass::Option, exchange: "CBOE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "NVDA", name: "NVDA Option Underlying", asset_class: AssetClass::Option, exchange: "CBOE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "TSLA", name: "TSLA Option Underlying", asset_class: AssetClass::Option, exchange: "CBOE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "MSFT", name: "MSFT Option Underlying", asset_class: AssetClass::Option, exchange: "CBOE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "AMZN", name: "AMZN Option Underlying", asset_class: AssetClass::Option, exchange: "CBOE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "AMD", name: "AMD Option Underlying", asset_class: AssetClass::Option, exchange: "CBOE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "META", name: "META Option Underlying", asset_class: AssetClass::Option, exchange: "CBOE", is_fractional: false, min_lot_size: 1.0 },
    Instrument { symbol: "IWM", name: "IWM Option Underlying", asset_class: AssetClass::Option, exchange: "CBOE", is_fractional: false, min_lot_size: 1.0 },
];

pub fn get_instrument(symbol: &str) -> Option<&'static Instrument> {
    INSTRUMENT_UNIVERSE.iter().find(|inst| inst.symbol == symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn universe_has_all_categories() {
        assert!(INSTRUMENT_UNIVERSE.len() >= 70);
        let crypto = get_instrument("BTC/USD").unwrap();
        assert_eq!(crypto.asset_class, AssetClass::Crypto);
        assert!(crypto.is_fractional);

        let indian = get_instrument("RELIANCE.NS").unwrap();
        assert_eq!(indian.asset_class, AssetClass::IndianEquity);

        let japanese = get_instrument("7203.T").unwrap();
        assert_eq!(japanese.asset_class, AssetClass::JapaneseEquity);
    }
}
