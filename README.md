

# Solana Trading Bot

A high-performance Rust-based trading bot for the Solana blockchain, designed to trade multiple meme coins or Solana tokens (e.g., BONK, WIF, SOL) using a combination of trading strategies: **sniping**, **grid trading**, and **trend following**, with integrated **stop-loss mechanisms** for risk management. The bot leverages Telegram for real-time trade monitoring and control, allowing users to start/stop strategies, check balances, and adjust parameters via commands. Built with Rust for speed and safety, it interacts with Solana’s blockchain using `solana-sdk` and connects to decentralized exchanges (DEXs) like Raydium and Jupiter for trading.

## Features

- **Multi-Token Trading**: Trades multiple Solana-based tokens concurrently (e.g., BONK, WIF, SOL).
- **Trading Strategies**:
  - **Sniping**: Buys new tokens at launch on Raydium, targeting early price pumps with profit targets (e.g., 10%) and stop-loss (e.g., 5%).
  - **Grid Trading**: Places buy/sell orders at fixed price intervals, profiting from volatility within a range, with stop-loss to exit if the market trends out of range.
  - **Trend Following**: Uses RSI (Relative Strength Index) to buy on uptrends, with trailing stop-loss to lock in profits.
- **Stop-Loss Mechanisms**:
  - **Fixed Stop-Loss**: Sells if price drops below a set percentage (e.g., 5% below entry).
  - **Trailing Stop-Loss**: Adjusts stop-loss upward as price rises (e.g., 5% below peak).
  - **Time-Based Stop-Loss**: Sells after a set time (e.g., 10 minutes for sniping) if no profit.
- **Telegram Integration**:
  - Real-time notifications for trades, stop-loss triggers, and price updates.
  - Commands: `/start`, `/stop`, `/balance`, `/status`, `/set_params` for controlling the bot.
  - Restricted to authorized Telegram user ID for security.
- **Performance**: Built in Rust for low-latency execution, critical for sniping and high-frequency trading.
- **Security**: Uses dedicated wallet, environment variables for sensitive data, and optional anti-MEV protection via QuickNode/Helius.

## Complexities and Design Considerations

### Architecture
- **Modular Design**: The bot is split into modules for wallet management (`wallet.rs`), price feeds (`price_feed.rs`), strategies (`sniper.rs`, `grid.rs`, `trend.rs`, `stop_loss.rs`), and Telegram integration (`telegram.rs`), ensuring maintainability and scalability.
- **Concurrency**: Uses `tokio` for asynchronous tasks, running strategies for each token in parallel to handle multiple tokens (e.g., BONK, WIF) simultaneously.
- **State Management**: A `HashMap` in `main.rs` tracks active strategies per token, with `tokio::mpsc` channels for processing Telegram commands.
- **Price Feeds**: Integrates with Jupiter API for real-time prices and Raydium API for new pool detection, with WebSocket support planned for optimization.
- **Stop-Loss Coordination**: Each strategy shares a `StopLoss` struct, running in a separate `tokio` task to monitor prices and trigger sells, with notifications sent to Telegram.

### Technical Challenges
- **Latency**: Solana’s high throughput requires low-latency RPCs (e.g., QuickNode) to avoid missed trades, especially for sniping new pools.
- **API Reliability**: External APIs (Jupiter, Raydium) may have rate limits or downtime; caching with Redis is recommended for production.
- **Meme Coin Volatility**: Meme coins like BONK or WIF can drop 90%+ in minutes, necessitating tight stop-loss settings and real-time Telegram alerts.
- **Rug Pull Risks**: New tokens may be scams; the bot filters for burned liquidity or locked tokens (via Raydium API) but requires manual verification.
- **Transaction Costs**: While Solana fees are low (~$0.0001/tx), frequent trades (e.g., grid trading) can accumulate costs, tracked via Telegram balance checks.
- **Security**: Private keys are stored in `.env`, but production systems should use a secure vault (e.g., AWS Secrets Manager). Telegram commands are restricted to a single user ID.

### Trade-offs
- **Rust vs. JavaScript**: Rust was chosen for performance and native Solana integration, but it has a steeper learning curve than JavaScript/TypeScript. JavaScript may be easier for rapid prototyping but sacrifices speed.
- **Custom vs. Pre-Built Bots**: Building a custom bot offers flexibility but requires coding expertise. Pre-built bots like BONKbot or Trojan are simpler but less customizable.
- **DEX vs. CEX**: The bot focuses on DEXs (Raydium, Jupiter) for meme coin trading, but centralized exchanges (e.g., Binance) could be added for SOL trading with additional API integrations.

## Prerequisites
- **Rust**: Install via `rustup` (https://rustup.rs/).
- **Solana CLI**: Install for keypair management (`cargo install solana-cli`).
- **Solana Wallet**: Create a wallet with SOL (e.g., via `solana-keygen new`) for fees and trading capital.
- **RPC Endpoint**: Use QuickNode or Helius for low-latency Solana access (free tier for testing).
- **Telegram Bot**: Create via `@BotFather` to get a bot token. Get your user ID via `@userinfobot`.
- **Dependencies**: Install via `Cargo.toml` (see below).

## Installation

1. **Clone the Repository**:
   ```bash
   git clone <repository_url>
   cd solana-trading-bot
   ```

2. **Set Up Environment**:
   Create a `.env` file in the project root:
   ```env
   WALLET_PRIVATE_KEY=your_base58_private_key_here
   RPC_ENDPOINT=https://api.mainnet-beta.solana.com # Replace with QuickNode/Helius
   JUPITER_API=https://quote-api.jup.ag/v6/quote
   RAYDIUM_POOL_API=https://api.raydium.io/v2/amm/pools
   TELEGRAM_BOT_TOKEN=your_bot_token_here
   TELEGRAM_USER_ID=your_user_id_here
   ```

3. **Install Dependencies**:
   ```bash
   cargo build --release
   ```

4. **Run the Bot**:
   ```bash
   cargo run --release
   ```

## Usage

### Telegram Commands
Interact with the bot via Telegram using the following commands:
- **`/start <token_mint>`**: Start sniping, grid, and trend strategies for a token (e.g., `/start DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263` for BONK).
- **`/stop <token_mint>`**: Stop all strategies for a token (e.g., `/stop EKpQGSJtjMFqKZ9u4uhkkR3eFfrk7unuZHKtvsH7BVvb` for WIF).
- **`/balance <token_mint>`**: Check wallet balance for a token (e.g., `/balance SOL...`).
- **`/status`**: List active tokens and strategies.
- **`/set_params <token_mint> <strategy> <key> <value>`**: Adjust strategy parameters (e.g., `/set_params BONK... sniper profit_target 0.2` for 20% profit target, `/set_params WIF... grid grid_levels 0.000018,0.000019,0.00002`).

### Example Workflow
1. Fund your Solana wallet with 1 SOL, split across tokens (e.g., 0.33 SOL for BONK, WIF, SOL).
2. Send `/start DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263` to start trading BONK.
3. Receive Telegram notifications: “Sniped BONK at 0.00002 SOL” or “Placed buy order for WIF at 0.000019”.
4. Monitor stop-loss triggers: “Stop-loss triggered for SOL at 150”.
5. Adjust parameters: `/set_params BONK... trend rsi_threshold 25`.
6. Check balances: `/balance WIF...` returns “Balance for WIF: 100000 tokens”.
7. Stop trading: `/stop SOL...`.

### Supported Tokens
- **BONK**: `DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263`
- **WIF**: `EKpQGSJtjMFqKZ9u4uhkkR3eFfrk7unuZHKtvsH7BVvb`
- **SOL**: Native SOL or wrapped SOL mint (e.g., `So11111111111111111111111111111111111111112`)
- Add more tokens in `main.rs` under the `tokens` vector.

## Strategies

### Sniping
- **Purpose**: Targets new token launches on Raydium, buying instantly to capture early price pumps.
- **Parameters**:
  - Profit target: Default 10% (adjustable via `/set_params <token> sniper profit_target <value>`).
  - Stop-loss: Fixed (5%) and trailing (5%), with time-based stop-loss (10 minutes) for rug-pull protection.
- **Complexity**: Monitors Raydium pools via API, requires low-latency RPC to snipe before others.

### Grid Trading
- **Purpose**: Places buy/sell orders at fixed price intervals (e.g., $0.000018-$0.000021), profiting from volatility.
- **Parameters**:
  - Grid levels: Default `[0.000018, 0.000019, 0.00002, 0.000021]` (adjustable via `/set_params <token> grid grid_levels <comma_separated_values>`).
  - Amount per order: 1000 tokens.
  - Stop-loss: Fixed (5%) and trailing (5%).
- **Complexity**: Balances multiple orders per token, requiring efficient transaction batching.

### Trend Following
- **Purpose**: Uses RSI to buy on uptrends (RSI < 30), selling at profit or trailing stop-loss.
- **Parameters**:
  - RSI period: 14 days.
  - RSI threshold: Default 30 (adjustable via `/set_params <token> trend rsi_threshold <value>`).
  - Stop-loss: Fixed (5%) and trailing (5%).
- **Complexity**: Requires historical price data for RSI calculation, with Telegram notifications for buy signals.

## Security Considerations
- **Wallet Security**: Use a dedicated wallet for the bot, not your main wallet. Store private keys in `.env` or a secure vault (e.g., AWS Secrets Manager).
- **Telegram Security**: Commands are restricted to the `TELEGRAM_USER_ID` specified in `.env`.
- **Anti-MEV**: Use QuickNode or Helius with anti-MEV protection to prevent frontrunning.
- **Network Security**: Run on a secure machine with a VPN and firewall. Avoid storing private keys in cloud storage.

## Optimization Tips
- **RPC Endpoint**: Use QuickNode ($9/month) or Helius for low-latency Solana access to minimize missed trades.
- **Caching**: Add Redis (`redis` crate) to cache price data, reducing API calls.
- **WebSocket**: Implement WebSocket connections for Raydium pool monitoring to improve sniping speed.
- **Backtesting**: Simulate strategies with historical data from DexScreener or Raydium APIs before live trading.
- **Capital Allocation**: Divide SOL across tokens (e.g., 0.33 SOL per token for 3 tokens) to diversify risk.

## Testing
1. **Devnet Testing**:
   - Update `.env` with `RPC_ENDPOINT=https://api.devnet.solana.com`.
   - Use fake SOL to test strategies and Telegram commands.
2. **Backtesting**:
   - Use `ta` crate to simulate RSI and grid strategies on historical data.
3. **Live Testing**:
   - Start with 0.1 SOL per token on mainnet.
   - Monitor Telegram for trade notifications and verify on Solscan.

## Deployment
- **Local**:
  - Run on a secure PC: `cargo run --release`.
- **Cloud**:
  - Deploy with Docker on AWS EC2 or DigitalOcean:
    ```dockerfile
    FROM rust:latest
    WORKDIR /app
    COPY . .
    RUN cargo build --release
    CMD ["./target/release/solana-trading-bot"]
    ```
  - Build and run:
    ```bash
    docker build -t solana-trading-bot .
    docker run --env-file .env solana-trading-bot
    ```

## Risks
- **Volatility**: Meme coins can lose 90%+ of value; Telegram stop-loss alerts help mitigate losses.
- **Rug Pulls**: New tokens may be scams; use `/status` to monitor active tokens and verify pool legitimacy via Raydium API.
- **Fees**: Solana fees are low (~$0.0001/tx), but frequent grid trading can accumulate costs, tracked via `/balance`.
- **Bugs**: Test thoroughly to avoid missed trades or incorrect command handling.
- **Legal**: Ensure compliance with local crypto trading regulations.

## Future Improvements


## Contributing
Contributions are welcome! Submit pull requests to enhance strategies, add commands, or improve performance. Join the Solana developer Discord for community support.

## License
MIT License