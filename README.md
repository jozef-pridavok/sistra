# Sistra - Cryptocurrency Trading Bot

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Sistra is an automated cryptocurrency trading bot written in Rust that implements multiple trading strategies for Bitcoin and other cryptocurrencies. The bot supports both live trading on centralized exchanges (CEX) and backtesting through historical data simulation.

## Features

- **Multiple Trading Strategies**:
  - EMA (Exponential Moving Average) Crossover
  - RSI (Relative Strength Index) based trading
  - Grid Trading
  - Buy the Dip strategy
  - Take Profit/Buyback strategy
  - Combined strategy with weighted signals

- **Exchange Support**:
  - KuCoin
  - OKX
  - Simulation mode with historical data

- **Risk Management**:
  - Configurable take profit and stop loss levels
  - Position sizing and allocation controls
  - Performance fee tracking
  - Portfolio balance management

- **Additional Features**:
  - Fear & Greed Index integration
  - Telegram notifications
  - Comprehensive logging
  - Backtesting capabilities
  - Real-time market data analysis

## Quick Start

### Prerequisites

- Rust 1.80 or higher
- Valid API credentials for supported exchanges (for live trading)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/sistra.git
cd sistra
```

2. Build the project:
```bash
cargo build --release
```

3. Copy and configure the settings:
```bash
cp config.toml.example config.toml
# Edit config.toml with your preferences and API credentials
```

### Configuration

The bot is configured through a `config.toml` file. Key parameters include:

```toml
# Basic settings
coin = "Bitcoin"                # Trading pair
period = 365                    # Analysis period in days
initial_btc = 0.5              # Initial BTC amount
initial_usd = 50_000           # Initial USD amount
allocation = 0.1               # Allocation percentage (10%)

# Risk management
take_profit_btc = 0.1          # Take profit threshold (10%)
stop_lose_btc = 0.3            # Stop loss threshold (30%)

# Strategy selection
strategy = "Combined"           # Strategy type
use_fear_index = true          # Use Fear & Greed Index

# Exchange settings
cex = "simulate"               # Exchange: "kucoin", "okx", or "simulate"
simulate_file = "./data/data_btc.json"  # Historical data for simulation
```

### Running the Bot

For simulation/backtesting:
```bash
cargo run
```

For live trading, ensure your API credentials are properly configured in `config.toml`.

## Trading Strategies

### EMA Crossover

Generates buy signals when short-term EMA crosses above long-term EMA, and sell signals on the opposite crossover.

**Description:**

Trend Following (e.g., EMA Crossover):
Tracks the crossing of exponential moving averages (e.g., 9-day and 21-day EMA).
When the faster EMA moves above the slower one, it signals a buy; when it moves below, it signals a sell.

- Advantage: Simple and historically effective on trending markets.
- Disadvantage: Can produce many false signals in sideways markets.

### RSI Strategy

**Buy Signal**: RSI < oversold threshold (default: 25)
**Sell Signal**: RSI > overbought threshold (default: 75)

**Description:**

RSI Based Strategy:
Buys when the Relative Strength Index (RSI) falls below a certain value (e.g., 25, indicating oversold conditions).
Sells when RSI rises above a certain value (e.g., 75, indicating overbought conditions).

- Advantage: Simple and popular.
- Disadvantage: RSI can give weak signals during strong trends.

### Grid Trading

Creates a price grid with multiple buy/sell levels around the current price, profiting from market volatility within a range.

**Description:**

Grid Trading:
Divides the price interval into several bands (the "grid").
Buys when the price drops below a certain level, sells when it rises above a certain level.

- Advantage: Works well in sideways markets where the price fluctuates within a range.
- Disadvantage: In sharp declines, you may end up holding a lot of assets bought at high prices; in sharp rises, you may sell everything too early.

### Buy the Dip

Automatically purchases during significant price drops (configurable percentage).

**Description:**

Buy the Dip (Averaging):
Regularly (weekly/monthly) buys BTC, or buys more during strong drops (e.g., >10% in a day).

- Advantage: Very simple, effective for long-term accumulation.
- Disadvantage: Profits mainly during long-term growth, not active trading.

### Take Profit/Buyback

Sells part of BTC when a certain profit is reached (e.g., +10%), buys part back during a drop (e.g., -10%).

- Advantage: Combines profit-taking and averaging purchases.


### Combined Strategy

Weights multiple strategies and generates signals based on consensus thresholds, providing more robust decision-making.

## Performance Tracking

The bot includes comprehensive performance tracking:
- Portfolio balance monitoring
- Performance fee calculation
- Trade execution logging
- Profit/loss analysis
- Telegram notifications for important events

## Project Structure

```
src/
├── main.rs          # Application entry point
├── config.rs        # Configuration management
├── strategy.rs      # Trading strategy implementations
├── executor.rs      # Trade execution logic
├── balance.rs       # Portfolio balance tracking
├── order.rs         # Order management
├── signal.rs        # Trading signal generation
├── fear_greed.rs    # Fear & Greed Index integration
├── telegram.rs      # Telegram notifications
├── logger.rs        # Logging utilities
└── cex/            # Exchange integrations
    ├── kucoin.rs
    ├── okx.rs
    └── simulate.rs
```

## Risk Disclaimer

**⚠️ Important Warning**: Cryptocurrency trading involves substantial risk of loss. This bot is provided for educational and research purposes. Always:

- Test thoroughly in simulation mode before live trading
- Use only funds you can afford to lose
- Monitor the bot's performance regularly
- Understand the strategies being employed
- Keep your API keys secure

The authors are not responsible for any financial losses incurred through the use of this software.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For questions or support, please open an issue on GitHub.

---

**Disclaimer**: This software is for educational purposes only. Trading cryptocurrencies carries significant financial risk. Use at your own discretion and risk.
