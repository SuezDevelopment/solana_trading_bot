use rusqlite::{ Connection, Result, params};
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
            []
        )?;
        Ok(TradeLog { conn })
    }

    pub fn log_trade(&self, token_mint: &str, action: &str, price: f64, amount: f64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO trades (token_mint, action, price, amount, timestamp) VALUES (?, ?, ?, ?, ?)",
            [token_mint, action, &price.to_string(), &amount.to_string(), &Utc::now().to_rfc3339()]
        )?;
        Ok(())
    }

    pub fn get_trades(
        &self,
        token_mint: &str,
        limit: i64
    ) -> Result<Vec<(String, String, f64, f64, String)>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT token_mint, action, price, amount, timestamp FROM trades WHERE token_mint = ? ORDER BY timestamp DESC LIMIT ?"
        )?;
        let rows = stmt.query_map(params![token_mint, limit], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?;
        let trades = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(trades)
    }

    pub fn calculate_profit(
        &self,
        token_mint: &str,
        current_price: f64
    ) -> Result<(f64, f64), rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT action, price, amount FROM trades WHERE token_mint = ? ORDER BY timestamp"
        )?;
        let trades = stmt
            .query_map(params![token_mint], |row| {
                Ok((
                    row.get(0)?, // action (String)
                    row.get(1)?, // price (f64)
                    row.get(2)?, // amount (f64)
                ))
            })?
            .collect::<Result<Vec<(String, f64, f64)>, _>>()?;

        let mut total_cost = 0.0;
        let mut total_amount = 0.0;
        for (action, price, amount) in trades {
            if action == "buy" {
                total_cost += price * amount;
                total_amount += amount;
            } else if action == "sell" {
                total_cost -= price * amount;
                total_amount -= amount;
            }
        }
        let current_value = total_amount * current_price;
        let profit = current_value - total_cost;
        let percentage = if total_cost > 0.0 { (profit / total_cost) * 100.0 } else { 0.0 };
        Ok((profit, percentage))
    }
}
