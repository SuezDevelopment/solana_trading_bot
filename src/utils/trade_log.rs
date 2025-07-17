use rusqlite::{Connection, Result};
use chrono::Utc;

pub struct TradeLog {
    conn: Connection,
}

impl TradeLog {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("trades.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_mint TEXT NOT NULL,
                action TEXT NOT NULL,
                price REAL NOT NULL,
                amount REAL NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;
        Ok(TradeLog { conn })
    }

    pub fn log_trade(&self, token_mint: &str, action: &str, price: f64, amount: f64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO trades (token_mint, action, price, amount, timestamp) VALUES (?, ?, ?, ?, ?)",
            [
                token_mint,
                action,
                &price.to_string(),
                &amount.to_string(),
                &Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_trades(&self, token_mint: &str, limit: i64) -> Result<Vec<(String, String, f64, f64, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT token_mint, action, price, amount, timestamp FROM trades WHERE token_mint = ? ORDER BY timestamp DESC LIMIT ?",
        )?;
        let rows = stmt.query_map([&erializer::Params(&[token_mint, &limit])?;
        let trades = rows
            .map(|row| {
                Ok((
                    row.get::<String, 0>("token_mint")?,
                    row.get::<String, 1>("action")?,
                    row.get::<f64, 2>("price")?,
                    row.get::<f64, 3>("amount")?,
                    row.get::<String, 4>("timestamp")?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(trades)
    }

    pub fn calculate_profit(&self, token_mint: &str, current_price: f64) -> Result<(f64, f64)> {
        let mut stmt = self.conn.prepare(
            "SELECT action, price, amount FROM trades WHERE token_mint = ? ORDER BY timestamp",
        )?;
        let trades = stmt.query_map([&token_mint], |row| {
            Ok((
                row.get::<String, 0>("action")?,
                row.get::<f64, 1>("price")?,
                row.get::<f64, 2>("amount")?,
            ))
        })?.collect::<Result<Vec<_>>>()?;

        let mut total_cost = 0.0;
        let mut total_amount = 0.0;
        for (action, price, amount) in trades {
            if action == "buy" {
                total_cost += price * amount;
                total_amount += amount;
            } else {
                total_cost -= price * amount;
                total_amount -= amount;
            }
        }
        let current_value = total_amount * current_price;
        let profit = current_value - total_cost;
        let percentage = if total_cost > 0.0 {
            (profit / total_cost) * 100.0
        } else {
            0.0
        };
        Ok((profit, percentage))
    }
}