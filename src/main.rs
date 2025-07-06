use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use fedimint_core::invite_code::InviteCode;
use fedimint_ln_common::lightning_invoice::Bolt11Invoice;
use tracing_subscriber::EnvFilter;

mod wallet;
use wallet::FedimintWallet;

#[derive(Parser)]
#[command(name = "simple-fedimint-wallet")]
#[command(about = "An interactive Fedimint wallet", long_about = None)]
struct Cli {
    /// Federation invite code (can be provided via FEDIMINT_INVITE_CODE env var)
    #[arg(short, long, env = "FEDIMINT_INVITE_CODE")]
    invite_code: String,

    /// Data directory for wallet storage
    #[arg(short, long, default_value = "./wallet-data")]
    data_dir: PathBuf,
}

fn print_menu() {
    println!("\n\nSimple Fedimint Wallet");
    println!("=====================\n");
    println!("1. Get Wallet Balance");
    println!("2. Create a Lightning Invoice");
    println!("3. Pay a Lightning Invoice");
    println!("4. Await Invoice Payment");
    println!("5. List Federation Gateways");
    println!("6. Exit");
    println!("\nSelect an option: ");
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
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
    println!("Initializing wallet...");
    let wallet = FedimintWallet::new(invite_code, cli.data_dir).await?;
    println!("Wallet initialized successfully!\n");

    loop {
        print_menu();
        let choice = get_user_input("");

        match choice.as_str() {
            "1" => match wallet.get_balance().await {
                Ok(balance) => {
                    println!(
                        "\nBalance: {} millisatoshis ({} sats)",
                        balance.msats,
                        balance.msats / 1000
                    );
                }
                Err(e) => println!("\nError getting balance: {}", e),
            },
            "2" => {
                let amount_str = get_user_input("\nEnter amount in millisatoshis: ");
                match amount_str.parse::<u64>() {
                    Ok(amount) => {
                        let description = get_user_input(
                            "Enter invoice description (or press Enter for default): ",
                        );
                        let description = if description.is_empty() {
                            "Fedimint wallet payment".to_string()
                        } else {
                            description
                        };

                        match wallet.create_invoice(amount, description).await {
                            Ok(invoice_info) => {
                                println!("\nInvoice created!");
                                println!("Lightning invoice: {}", invoice_info.invoice);
                                println!("Operation ID: {}", invoice_info.operation_id.fmt_full());
                                println!(
                                    "\nShare the invoice for payment. Use option 4 with the operation ID to check payment status."
                                );
                            }
                            Err(e) => println!("\nError creating invoice: {}", e),
                        }
                    }
                    Err(_) => println!("\nInvalid amount. Please enter a number."),
                }
            }
            "3" => {
                let invoice_str = get_user_input("\nEnter Lightning invoice: ");
                match Bolt11Invoice::from_str(&invoice_str) {
                    Ok(invoice) => match wallet.pay_invoice(invoice).await {
                        Ok(payment_info) => {
                            println!("\nPayment initiated!");
                            println!("Contract ID: {}", payment_info.contract_id);
                            println!("Fee: {} msats", payment_info.fee.msats);
                            println!("Payment type: {:?}", payment_info.payment_type);
                        }
                        Err(e) => println!("\nError paying invoice: {}", e),
                    },
                    Err(e) => println!("\nInvalid invoice: {}", e),
                }
            }
            "4" => {
                let operation_id = get_user_input("\nEnter operation ID: ");
                match operation_id.parse() {
                    Ok(op_id) => {
                        println!("\nWaiting for payment...");
                        match wallet.await_invoice_payment(op_id).await {
                            Ok(_) => println!("Invoice has been paid!"),
                            Err(e) => println!("Error awaiting payment: {}", e),
                        }
                    }
                    Err(e) => println!("\nInvalid operation ID: {}", e),
                }
            }
            "5" => match wallet.list_gateways().await {
                Ok(gateways) => {
                    if gateways.is_empty() {
                        println!("\nNo gateways found in the federation.");
                    } else {
                        println!("\nFederation Gateways:");
                        println!("===================");
                        for gateway in gateways {
                            println!("\nGateway ID: {}", gateway.info.gateway_id);
                            println!("  TTL: {} seconds", gateway.ttl.as_secs());
                            println!(
                                "  Route hints: {} available",
                                gateway.info.route_hints.len()
                            );
                            println!("  Lightning alias: {}", gateway.info.lightning_alias);
                            println!("  API endpoint: {}", gateway.info.api);
                            println!("  Node public key: {}", gateway.info.node_pub_key);
                            if gateway.info.supports_private_payments {
                                println!("  Supports private payments: Yes");
                            } else {
                                println!("  Supports private payments: No");
                            }
                        }
                    }
                }
                Err(e) => println!("\nError listing gateways: {}", e),
            },
            "6" => {
                println!("\nExiting wallet...");
                break;
            }
            _ => {
                println!("\nInvalid option. Please select 1-6.");
            }
        }
    }

    Ok(())
}
