use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};

pub const ELF: &[u8] = include_elf!("sp1-financial-program");

#[derive(Serialize, Deserialize, Clone)]
pub enum FinancialOperation {
    HealthFactor {
        collateral_value: i128,
        debt_value: i128,
        liquidation_threshold_bps: i128,
    },
    CompoundInterest {
        principal: i128,
        rate_bps: i128,
        periods: u32,
    },
    SwapOutput {
        reserve_in: i128,
        reserve_out: i128,
        amount_in: i128,
        fee_bps: i128,
    },
    LiquidationPrice {
        collateral_amount: i128,
        debt_value: i128,
        liquidation_threshold_bps: i128,
    },
    SharePrice {
        total_assets: i128,
        total_supply: i128,
    },
}

#[derive(Serialize, Deserialize)]
pub struct FinancialResult {
    pub value: i128,
    pub scale: u32,
}

#[derive(Parser)]
#[command(name = "sp1-financial")]
#[command(about = "Generate ZK proofs for financial calculations using Keystone")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    HealthFactor {
        #[arg(long)]
        collateral: i128,
        #[arg(long)]
        debt: i128,
        #[arg(long, default_value = "8000")]
        threshold_bps: i128,
        #[arg(long)]
        prove: bool,
    },
    CompoundInterest {
        #[arg(long)]
        principal: i128,
        #[arg(long)]
        rate_bps: i128,
        #[arg(long)]
        periods: u32,
        #[arg(long)]
        prove: bool,
    },
    SwapOutput {
        #[arg(long)]
        reserve_in: i128,
        #[arg(long)]
        reserve_out: i128,
        #[arg(long)]
        amount_in: i128,
        #[arg(long, default_value = "30")]
        fee_bps: i128,
        #[arg(long)]
        prove: bool,
    },
    SharePrice {
        #[arg(long)]
        total_assets: i128,
        #[arg(long)]
        total_supply: i128,
        #[arg(long)]
        prove: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::HealthFactor {
            collateral,
            debt,
            threshold_bps,
            prove,
        } => {
            let op = FinancialOperation::HealthFactor {
                collateral_value: collateral,
                debt_value: debt,
                liquidation_threshold_bps: threshold_bps,
            };
            run_operation(op, prove)?;
        }
        Commands::CompoundInterest {
            principal,
            rate_bps,
            periods,
            prove,
        } => {
            let op = FinancialOperation::CompoundInterest {
                principal,
                rate_bps,
                periods,
            };
            run_operation(op, prove)?;
        }
        Commands::SwapOutput {
            reserve_in,
            reserve_out,
            amount_in,
            fee_bps,
            prove,
        } => {
            let op = FinancialOperation::SwapOutput {
                reserve_in,
                reserve_out,
                amount_in,
                fee_bps,
            };
            run_operation(op, prove)?;
        }
        Commands::SharePrice {
            total_assets,
            total_supply,
            prove,
        } => {
            let op = FinancialOperation::SharePrice {
                total_assets,
                total_supply,
            };
            run_operation(op, prove)?;
        }
    }

    Ok(())
}

fn run_operation(operation: FinancialOperation, generate_proof: bool) -> Result<()> {
    let client = ProverClient::from_env();

    let mut stdin = SP1Stdin::new();
    stdin.write(&operation);

    if generate_proof {
        println!("Generating ZK proof...");
        let (pk, vk) = client.setup(ELF);
        let proof = client.prove(&pk, &stdin).groth16().run()?;

        let result: FinancialResult = proof.public_values.read();
        print_result(&operation, &result);

        println!("\nVerifying proof...");
        client.verify(&proof, &vk)?;
        println!("Proof verified successfully");

        let proof_bytes = bincode::serialize(&proof)?;
        println!("Proof size: {} bytes", proof_bytes.len());
    } else {
        println!("Executing without proof (simulation)...");
        let (output, _report) = client.execute(ELF, &stdin).run()?;

        let result: FinancialResult = output.read();
        print_result(&operation, &result);
    }

    Ok(())
}

fn print_result(operation: &FinancialOperation, result: &FinancialResult) {
    let scaled_value = result.value as f64 / 10f64.powi(result.scale as i32);

    match operation {
        FinancialOperation::HealthFactor { .. } => {
            println!("Health Factor: {:.6}", scaled_value);
            if scaled_value < 1.0 {
                println!("Status: LIQUIDATABLE");
            } else {
                println!("Status: HEALTHY");
            }
        }
        FinancialOperation::CompoundInterest { principal, .. } => {
            let principal_f = *principal as f64 / 1e18;
            println!("Principal: {:.6}", principal_f);
            println!("Final Amount: {:.6}", scaled_value);
            println!("Interest Earned: {:.6}", scaled_value - principal_f);
        }
        FinancialOperation::SwapOutput { amount_in, .. } => {
            let input = *amount_in as f64 / 1e18;
            println!("Input Amount: {:.6}", input);
            println!("Output Amount: {:.6}", scaled_value);
        }
        FinancialOperation::SharePrice { .. } => {
            println!("Share Price: {:.6}", scaled_value);
        }
        FinancialOperation::LiquidationPrice { .. } => {
            println!("Liquidation Price: {:.6}", scaled_value);
        }
    }

    println!("\nRaw result: {} (scale: {})", result.value, result.scale);
}
