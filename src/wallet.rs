use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use fedimint_client::{Client, ClientHandleArc, ClientModuleInstance, OperationId};
use fedimint_core::Amount;
use fedimint_core::invite_code::InviteCode;
use fedimint_ln_client::{LightningClientModule, LnReceiveState};
use fedimint_ln_common::LightningGatewayAnnouncement;
use fedimint_ln_common::lightning_invoice::{Bolt11Invoice, Description};
use futures::StreamExt;
use serde::Serialize;
use thiserror::Error;
use tracing::{debug, error, info, warn};

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Invoice amount is zero")]
    InvoiceAmountZero,

    #[error("Invoice canceled: {0}")]
    InvoiceCanceled(String),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Serialize)]
pub struct InvoiceInfo {
    pub operation_id: OperationId,
    pub invoice: String,
}

/// Helper functions for fedimint_client::Client
pub trait ClientExt {
    /// Attempt to get the first lightning client module instance.
    fn lightning(&self) -> anyhow::Result<ClientModuleInstance<LightningClientModule>>;
}

impl ClientExt for Client {
    fn lightning(&self) -> anyhow::Result<ClientModuleInstance<LightningClientModule>> {
        self.get_first_module::<LightningClientModule>()
    }
}

pub type OutgoingPaymentInfo = fedimint_ln_client::OutgoingLightningPayment;

pub struct FedimintWallet {
    client: ClientHandleArc,
}

impl FedimintWallet {
    /// Create a new wallet instance or recover an existing one
    pub async fn new(invite_code: InviteCode, data_dir: PathBuf) -> anyhow::Result<Self> {
        let (client, _) = build_client(Some(invite_code.clone()), Some(&data_dir)).await?;

        Ok(Self { client })
    }

    /// Get the current balance in millisatoshis
    pub async fn get_balance(&self) -> anyhow::Result<Amount> {
        Ok(self.client.get_balance().await)
    }

    /// Create a Lightning invoice
    pub async fn create_invoice(
        &self,
        amount_msat: u64,
        description: String,
    ) -> anyhow::Result<InvoiceInfo, WalletError> {
        if amount_msat == 0 {
            return Err(WalletError::InvoiceAmountZero);
        }

        let amount = Amount::from_msats(amount_msat);
        let desc = Description::new(description)
            .map_err(|e| WalletError::Other(anyhow::anyhow!("Invalid description: {}", e)))?;

        // Select the gateway with the longest TTL
        let gateways = self.list_gateways().await?;
        let gateway = gateways
            .into_iter()
            .max_by_key(|g| g.ttl)
            .context("No gateways found")?;

        let (operation_id, invoice, _) = self
            .client
            .lightning()?
            .create_bolt11_invoice(
                amount,
                fedimint_ln_common::lightning_invoice::Bolt11InvoiceDescription::Direct(&desc),
                None, // No expiry time
                (),
                Some(gateway.info),
            )
            .await?;

        info!("Created invoice: {} for {} msat", invoice, amount_msat);

        Ok(InvoiceInfo {
            operation_id,
            invoice: invoice.to_string(),
        })
    }

    /// Pay a Lightning invoice
    pub async fn pay_invoice(
        &self,
        invoice: Bolt11Invoice,
    ) -> anyhow::Result<OutgoingPaymentInfo, WalletError> {
        // Check balance if invoice has amount
        if let Some(amount_msat) = invoice.amount_milli_satoshis() {
            if amount_msat == 0 {
                return Err(WalletError::InvoiceAmountZero);
            }

            let balance = self.client.get_balance().await;
            if balance.msats < amount_msat {
                return Err(WalletError::InsufficientBalance);
            }
        }

        // Select the gateway with the longest TTL
        let gateways = self.list_gateways().await?;
        let gateway = gateways
            .into_iter()
            .max_by_key(|g| g.ttl)
            .context("No gateways found")?;

        info!("Paying invoice: {}", invoice);
        let payment = self
            .client
            .lightning()?
            .pay_bolt11_invoice(Some(gateway.info), invoice, ())
            .await?;

        Ok(payment)
    }

    /// Wait for an invoice to be paid
    pub async fn await_invoice_payment(
        &self,
        operation_id: OperationId,
    ) -> anyhow::Result<(), WalletError> {
        let operation_exists = self.client.operation_exists(operation_id).await;
        if !operation_exists {
            return Err(WalletError::Other(anyhow::anyhow!(
                "Operation does not exist"
            )));
        }

        let mut updates = self
            .client
            .lightning()?
            .subscribe_ln_receive(operation_id)
            .await
            .context("Failed to subscribe to invoice updates")?
            .into_stream();

        while let Some(update) = updates.next().await {
            match update {
                LnReceiveState::Funded => {
                    info!("Invoice paid!");
                    return Ok(());
                }
                LnReceiveState::Canceled { reason } => {
                    warn!("Invoice canceled: {reason:?}");
                    return Err(WalletError::InvoiceCanceled(reason.to_string()));
                }
                other => {
                    debug!("Invoice update: {:?}", other);
                }
            }
        }

        Err(WalletError::Other(anyhow::anyhow!(
            "No updates received for invoice with operation id: {}",
            operation_id.fmt_full()
        )))
    }

    /// List all available gateways in the federation
    pub async fn list_gateways(&self) -> anyhow::Result<Vec<LightningGatewayAnnouncement>> {
        // Update gateway cache first
        self.client.lightning()?.update_gateway_cache().await?;

        // Get all gateways
        let gateways = self.client.lightning()?.list_gateways().await;

        Ok(gateways)
    }
}

/// Build a Fedimint client
async fn build_client(
    invite_code: Option<InviteCode>,
    data_dir: Option<&PathBuf>,
) -> anyhow::Result<(ClientHandleArc, Option<InviteCode>)> {
    use fedimint_client::secret::{PlainRootSecretStrategy, RootSecretStrategy};
    use fedimint_core::db::Database;
    use fedimint_core::module::registry::ModuleRegistry;
    use fedimint_ln_client::LightningClientInit;
    use fedimint_mint_client::MintClientInit;
    use fedimint_wallet_client::WalletClientInit;

    let db = if let Some(data_dir) = data_dir {
        Database::new(
            fedimint_rocksdb::RocksDb::open(data_dir).await?,
            ModuleRegistry::default(),
        )
    } else {
        fedimint_core::db::mem_impl::MemDatabase::new().into()
    };

    let mut client_builder = Client::builder(db).await?;
    client_builder.with_module(MintClientInit);
    client_builder.with_module(LightningClientInit::default());
    client_builder.with_module(WalletClientInit::default());
    client_builder.with_primary_module_kind(fedimint_mint_client::KIND);

    let client_secret =
        Client::load_or_generate_client_secret(client_builder.db_no_decoders()).await?;
    let root_secret = PlainRootSecretStrategy::to_root_secret(&client_secret);

    let client = if Client::is_initialized(client_builder.db_no_decoders()).await {
        client_builder.open(root_secret).await?
    } else if let Some(invite_code) = &invite_code {
        let client_config = fedimint_api_client::api::net::Connector::default()
            .download_from_invite_code(invite_code)
            .await?;
        client_builder
            .join(root_secret, client_config.clone(), invite_code.api_secret())
            .await?
    } else {
        anyhow::bail!("Database not initialized and invite code not provided");
    };

    Ok((Arc::new(client), invite_code))
}
