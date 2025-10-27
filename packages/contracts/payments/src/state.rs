use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[cw_serde]
#[derive(Default)]
pub struct PendingPayments {
    payments: Vec<Coin>,
}

impl PendingPayments {
    /// Only safe way to add.
    pub fn add_payment(&mut self, payment: Coin) {
        // TODO: add some sanity check (payment.amount > 0)
        // check if there is an existing entry with this denom and combine them
        if let Some(i) = self.payments.iter().position(|p| p.denom == payment.denom) {
            self.payments[i].amount += payment.amount;
        } else {
            self.payments.push(payment);
        }
    }

    /// Makes payments readable but not writable
    pub fn balance(self) -> Vec<Coin> {
        self.payments
    }
}

/// Maps a telegram handle to a blockchain address
pub const OPEN_ACCOUNTS: Map<&str, Addr> = Map::new("open_accounts");
/// Maps a blockchain address to a telegram handle
pub const FUNDED_ACCOUNTS: Map<&Addr, String> = Map::new("funded_accounts");

/// Maps an unregistered telegram handle to a list of pending payments, only one
pub const PENDING_PAYMENTS: Map<&str, PendingPayments> = Map::new("pending_payments");

/// Which denoms we will accept for payments
pub const ALLOWED_DENOMS: Item<Vec<String>> = Item::new("allowed_denoms");

/// Only set if we take ServiceHandler interface
pub const SERVICE_MANAGER: Item<Addr> = Item::new("service_manager");
/// Only set in the test approach
pub const ADMIN: Item<Addr> = Item::new("admin");
