use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub protocol: String,
    pub asset: String,
    pub balance_usd: f64,
    pub is_collateral: bool,
    pub apy: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    pub total_collateral: f64,
    pub total_debt: f64,
    pub health_factor: f64,
    pub liquidation_distance: f64,
    pub positions: Vec<Position>,
}

pub async fn fetch_portfolio(wallet: &str) -> Result<PortfolioMetrics> {
    let (aave, compound, uniswap) = tokio::join!(
        fetch_aave_positions(wallet),
        fetch_compound_positions(wallet),
        fetch_uniswap_positions(wallet)
    );

    let mut positions = Vec::new();
    let mut total_collateral = 0.0;
    let mut total_debt = 0.0;

    if let Ok(aave_positions) = aave {
        for pos in aave_positions {
            if pos.is_collateral {
                total_collateral += pos.balance_usd;
            } else {
                total_debt += pos.balance_usd;
            }
            positions.push(pos);
        }
    }

    if let Ok(compound_positions) = compound {
        for pos in compound_positions {
            if pos.is_collateral {
                total_collateral += pos.balance_usd;
            } else {
                total_debt += pos.balance_usd;
            }
            positions.push(pos);
        }
    }

    if let Ok(uniswap_positions) = uniswap {
        for pos in uniswap_positions {
            total_collateral += pos.balance_usd;
            positions.push(pos);
        }
    }

    let health_factor = if total_debt > 0.0 {
        (total_collateral * 0.8) / total_debt
    } else {
        f64::MAX
    };

    let liquidation_distance = if health_factor < f64::MAX {
        ((health_factor - 1.0) / health_factor * 100.0).max(0.0)
    } else {
        100.0
    };

    Ok(PortfolioMetrics {
        total_collateral,
        total_debt,
        health_factor,
        liquidation_distance,
        positions,
    })
}

async fn fetch_aave_positions(wallet: &str) -> Result<Vec<Position>> {
    let query = r#"
        query($wallet: String!) {
            userReserves(where: { user: $wallet }) {
                reserve {
                    symbol
                    decimals
                    priceInUSD
                    liquidationThreshold
                }
                currentATokenBalance
                currentVariableDebt
            }
        }
    "#;

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.thegraph.com/subgraphs/name/aave/protocol-v3")
        .json(&serde_json::json!({
            "query": query,
            "variables": { "wallet": wallet.to_lowercase() }
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let mut positions = Vec::new();

    if let Some(reserves) = resp["data"]["userReserves"].as_array() {
        for reserve in reserves {
            let symbol = reserve["reserve"]["symbol"].as_str().unwrap_or("UNKNOWN");
            let decimals = reserve["reserve"]["decimals"].as_i64().unwrap_or(18) as u32;
            let price = reserve["reserve"]["priceInUSD"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let collateral_raw = reserve["currentATokenBalance"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let debt_raw = reserve["currentVariableDebt"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let divisor = 10_f64.powi(decimals as i32);

            if collateral_raw > 0.0 {
                positions.push(Position {
                    protocol: "Aave".to_string(),
                    asset: symbol.to_string(),
                    balance_usd: (collateral_raw / divisor) * price,
                    is_collateral: true,
                    apy: None,
                });
            }

            if debt_raw > 0.0 {
                positions.push(Position {
                    protocol: "Aave".to_string(),
                    asset: symbol.to_string(),
                    balance_usd: (debt_raw / divisor) * price,
                    is_collateral: false,
                    apy: None,
                });
            }
        }
    }

    Ok(positions)
}

async fn fetch_compound_positions(wallet: &str) -> Result<Vec<Position>> {
    let query = r#"
        query($wallet: String!) {
            accounts(where: { id: $wallet }) {
                positions {
                    balance
                    market {
                        inputToken {
                            symbol
                            decimals
                        }
                        inputTokenPriceUSD
                    }
                }
            }
        }
    "#;

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.thegraph.com/subgraphs/name/compound-finance/compound-v3")
        .json(&serde_json::json!({
            "query": query,
            "variables": { "wallet": wallet.to_lowercase() }
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let mut positions = Vec::new();

    if let Some(accounts) = resp["data"]["accounts"].as_array() {
        for account in accounts {
            if let Some(account_positions) = account["positions"].as_array() {
                for pos in account_positions {
                    let balance = pos["balance"]
                        .as_str()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);

                    if balance.abs() < 0.000001 {
                        continue;
                    }

                    let symbol = pos["market"]["inputToken"]["symbol"]
                        .as_str()
                        .unwrap_or("UNKNOWN");
                    let decimals = pos["market"]["inputToken"]["decimals"]
                        .as_i64()
                        .unwrap_or(18) as u32;
                    let price = pos["market"]["inputTokenPriceUSD"]
                        .as_str()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);

                    let divisor = 10_f64.powi(decimals as i32);
                    let balance_usd = (balance.abs() / divisor) * price;

                    positions.push(Position {
                        protocol: "Compound".to_string(),
                        asset: symbol.to_string(),
                        balance_usd,
                        is_collateral: balance > 0.0,
                        apy: None,
                    });
                }
            }
        }
    }

    Ok(positions)
}

async fn fetch_uniswap_positions(wallet: &str) -> Result<Vec<Position>> {
    let query = r#"
        query($wallet: String!) {
            positions(where: { owner: $wallet, liquidity_gt: "0" }) {
                liquidity
                token0 { symbol decimals }
                token1 { symbol decimals }
                depositedToken0
                depositedToken1
            }
            bundle(id: "1") { ethPriceUSD }
        }
    "#;

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3")
        .json(&serde_json::json!({
            "query": query,
            "variables": { "wallet": wallet.to_lowercase() }
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let eth_price = resp["data"]["bundle"]["ethPriceUSD"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(2000.0);

    let mut positions = Vec::new();

    if let Some(lp_positions) = resp["data"]["positions"].as_array() {
        for pos in lp_positions {
            let token0 = pos["token0"]["symbol"].as_str().unwrap_or("???");
            let token1 = pos["token1"]["symbol"].as_str().unwrap_or("???");

            let amount0 = pos["depositedToken0"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let amount1 = pos["depositedToken1"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let estimated_usd = (amount0 + amount1) * eth_price * 0.5;

            positions.push(Position {
                protocol: "Uniswap".to_string(),
                asset: format!("{}/{}", token0, token1),
                balance_usd: estimated_usd,
                is_collateral: true,
                apy: None,
            });
        }
    }

    Ok(positions)
}
