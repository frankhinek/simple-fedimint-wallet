use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap::{Parser, Subcommand};
use fedimint_core::invite_code::InviteCode;
use fedimint_ln_common::lightning_invoice::Bolt11Invoice;
use tracing_subscriber::EnvFilter;

mod wallet;
use wallet::FedimintWallet;

#[derive(Parser)]
#[command(name = "simple-fedimint-wallet")]
#[command(about = "A minimal Fedimint wallet implementation", long_about = None)]
struct Cli {
    /// Federation invite code
    #[arg(short, long, env = "FEDIMINT_INVITE_CODE")]
    invite_code: String,

    /// Data directory for wallet storage
    #[arg(short, long, default_value = "./wallet-data")]
    data_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show wallet balance
    Balance,

    /// Create a Lightning invoice
    Invoice {
        /// Amount in millisatoshis
        #[arg(short, long)]
        amount: u64,

        /// Invoice description
        #[arg(short, long, default_value = "Fedimint wallet payment")]
        description: String,
    },

    /// Pay a Lightning invoice
    Pay {
        /// Lightning invoice to pay
        invoice: String,
    },

    /// Wait for an invoice to be paid
    AwaitPayment {
        /// Operation ID from invoice creation
        operation_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    // Parse invite code
    let invite_code = InviteCode::from_str(&cli.invite_code)?;

    // Create or recover wallet
    let wallet = FedimintWallet::new(invite_code, cli.data_dir).await?;

    match cli.command {
        Commands::Balance => {
            let balance = wallet.get_balance().await?;
            println!(
                "Balance: {} millisatoshis ({} sats)",
                balance.msats,
                balance.msats / 1000
            );
        }

        Commands::Invoice {
            amount,
            description,
        } => {
            let invoice_info = wallet.create_invoice(amount, description).await?;
            println!("Invoice created!");
            println!("Lightning invoice: {}", invoice_info.invoice);
            println!("Operation ID: {}", invoice_info.operation_id.fmt_full());
            println!(
                "\nShare the invoice for payment. Use 'await-payment' command with the operation ID to check payment status."
            );
        }

        Commands::Pay { invoice } => {
            let invoice = Bolt11Invoice::from_str(&invoice)?;
            let payment_info = wallet.pay_invoice(invoice).await?;
            println!("Payment initiated!");
            println!("Contract ID: {}", payment_info.contract_id);
            println!("Fee: {} msats", payment_info.fee.msats);
            println!("Payment type: {:?}", payment_info.payment_type);
        }

        Commands::AwaitPayment { operation_id } => {
            let op_id = operation_id.parse()?;
            println!("Waiting for payment...");
            wallet.await_invoice_payment(op_id).await?;
            println!("Invoice has been paid!");
        }
    }

    Ok(())
}
