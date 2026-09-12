# Multi-Asset Trading Strategy Specification

---

## 1. Traded Universe Across Global Markets (10+ Symbols Each)

The bot supports multi-asset categorization with 10 benchmark symbols across each global market and instrument class:

### 1. Cryptocurrency (Alpaca Crypto)
1. **`BTC/USD`** — Bitcoin
2. **`ETH/USD`** — Ethereum
3. **`SOL/USD`** — Solana
4. **`DOGE/USD`** — Dogecoin
5. **`AVAX/USD`** — Avalanche
6. **`LINK/USD`** — Chainlink
7. **`LTC/USD`** — Litecoin
8. **`UNI/USD`** — Uniswap
9. **`BCH/USD`** — Bitcoin Cash
10. **`AAVE/USD`** — Aave

### 2. Indian Equities (National Stock Exchange - NSE)
1. **`RELIANCE.NS`** — Reliance Industries
2. **`TCS.NS`** — Tata Consultancy Services
3. **`HDFCBANK.NS`** — HDFC Bank
4. **`INFY.NS`** — Infosys
5. **`ICICIBANK.NS`** — ICICI Bank
6. **`HINDUNILVR.NS`** — Hindustan Unilever
7. **`ITC.NS`** — ITC Limited
8. **`SBIN.NS`** — State Bank of India
9. **`BHARTIARTL.NS`** — Bharti Airtel
10. **`LT.NS`** — Larsen & Toubro

### 3. Japanese Equities (Tokyo Stock Exchange - TSE)
1. **`7203.T`** — Toyota Motor Corporation
2. **`6758.T`** — Sony Group Corporation
3. **`9984.T`** — SoftBank Group
4. **`6861.T`** — Keyence Corporation
5. **`8035.T`** — Tokyo Electron
6. **`9432.T`** — Nippon Telegraph & Telephone (NTT)
7. **`6501.T`** — Hitachi Ltd
8. **`7974.T`** — Nintendo Co Ltd
9. **`8306.T`** — Mitsubishi UFJ Financial Group
10. **`6098.T`** — Recruit Holdings

### 4. Chinese & Hong Kong Equities (HKEX / SSE)
1. **`0700.HK`** — Tencent Holdings
2. **`9988.HK`** — Alibaba Group
3. **`3690.HK`** — Meituan
4. **`1810.HK`** — Xiaomi Corporation
5. **`1211.HK`** — BYD Company
6. **`0941.HK`** — China Mobile
7. **`0939.HK`** — China Construction Bank
8. **`2318.HK`** — Ping An Insurance
9. **`9618.HK`** — JD.com
10. **`600519.SS`** — Kweichow Moutai

### 5. European Equities (Euronext / LSE / DAX)
1. **`ASML.AS`** — ASML Holding (Netherlands)
2. **`MC.PA`** — LVMH Moët Hennessy (France)
3. **`SAP.DE`** — SAP SE (Germany)
4. **`SIE.DE`** — Siemens AG (Germany)
5. **`TTE.PA`** — TotalEnergies (France)
6. **`SHEL.L`** — Shell plc (United Kingdom)
7. **`AZN.L`** — AstraZeneca (United Kingdom)
8. **`SAN.PA`** — Sanofi (France)
9. **`AIR.PA`** — Airbus SE (France)
10. **`ALV.DE`** — Allianz SE (Germany)

### 6. Futures (CME / NYMEX / CBOT)
1. **`ES`** — E-mini S&P 500
2. **`NQ`** — E-mini Nasdaq 100
3. **`YM`** — E-mini Dow Jones
4. **`RTY`** — E-mini Russell 2000
5. **`CL`** — Crude Oil WTI
6. **`GC`** — Gold
7. **`SI`** — Silver
8. **`NG`** — Natural Gas
9. **`ZB`** — 30-Year U.S. Treasury Bond
10. **`ZN`** — 10-Year U.S. Treasury Note

### 7. Options (US Index & Equity Options)
1. **`SPY` Options** (S&P 500 ETF)
2. **`QQQ` Options** (Nasdaq 100 ETF)
3. **`AAPL` Options** (Apple)
4. **`NVDA` Options** (Nvidia)
5. **`TSLA` Options** (Tesla)
6. **`MSFT` Options** (Microsoft)
7. **`AMZN` Options** (Amazon)
8. **`AMD` Options** (AMD)
9. **`META` Options** (Meta Platforms)
10. **`IWM` Options** (Russell 2000 ETF)

---

## 2. The Trading Strategy: EMA Crossover with RSI Momentum Filter

### Strategy Name
**EMA Crossover + RSI Momentum Filter (Trend-Following with Exhaustion Gating)**

### Philosophy
Pure moving average crossovers often suffer from "whipsaws" (false breakouts) when entering late in a move or when prices are overextended. Adding the **Relative Strength Index (RSI)** ensures the bot only enters when momentum confirms the direction and is not already exhausted.

### Indicators Used
1. **Short-Term EMA (`EMA_20`)**: 20-period Exponential Moving Average on bar close.
2. **Long-Term EMA (`EMA_50`)**: 50-period Exponential Moving Average on bar close.
3. **Relative Strength Index (`RSI_14`)**: 14-period standard Wilder RSI.

### Mathematical Formulation

#### Exponential Moving Average:
$$k = \frac{2}{\text{period} + 1}$$
$$\text{EMA}_t = P_t \times k + \text{EMA}_{t-1} \times (1 - k)$$

#### Relative Strength Index:
$$\text{RS} = \frac{\text{Average Gain over 14 periods}}{\text{Average Loss over 14 periods}}$$
$$\text{RSI} = 100 - \frac{100}{1 + \text{RS}}$$

---

### Signal Generation Rules

| Signal | Condition | Rationale |
|---|---|---|
| **BUY** | $\text{EMA}_{20}$ crosses **above** $\text{EMA}_{50}$ **AND** $40 \le \text{RSI} \le 70$ | Bullish trend confirmed by momentum, without entering an overbought top ($> 70$). |
| **SELL** | $\text{EMA}_{20}$ crosses **below** $\text{EMA}_{50}$ **AND** $30 \le \text{RSI} \le 60$ | Bearish trend confirmed by momentum, without entering an oversold bounce ($< 30$). |
| **TAKE PROFIT EXIT** | In Long position **AND** $\text{RSI} \ge 80$ | Exits early into extreme parabolic exhaustion. |
| **HOLD** | No crossover or RSI filter blocks entry | Preserves capital; prevents whipsaw losses. |

---

## 3. Capital Allocation: 70% Net Worth Dynamic Sizing

Instead of fixed share quantities, position sizes are calculated dynamically based on real-time portfolio equity:

$$\text{Target Maximum Exposure} = \text{Portfolio Net Worth} \times 0.70$$

### Sizing Algorithm
1. **Total Net Worth** = $\text{Cash} + \sum (\text{Position Qty}_i \times \text{Current Price}_i)$.
2. **Current Total Exposure** = $\sum (\text{Position Qty}_i \times \text{Current Price}_i)$.
3. **Available Margin for New Position**:
   $$\text{Available} = \max(0, \text{Target Maximum Exposure} - \text{Current Total Exposure})$$
   *If $\text{Available} \le 0$, the Risk Manager **rejects** new Buy orders.*
4. **Per-Trade Allocation**:
   Divided among active target slots (e.g. max 5 positions):
   $$\text{Trade Allocation} = \min\left(\text{Available}, \frac{\text{Target Maximum Exposure}}{N_{\text{max}}}\right)$$
5. **Order Quantity**:
   $$\text{Order Qty} = \frac{\text{Trade Allocation}}{\text{Current Market Price}}$$
   *(Fractional precision applied for Crypto; rounded to integer lots for Equities and Contracts for Futures/Options).*

---

## 4. Broker Execution Routing

* **Alpaca API**: Executes US Equities, Cryptocurrencies, and US Options directly on Alpaca Paper/Live accounts.
* **Global Broker Adapter Layer (IBKR / Zerodha / Simulated)**: Provides routing interfaces for Indian (NSE), Japanese (TSE), Chinese (HKEX), European (Euronext/LSE), and Futures (CME) contracts.
