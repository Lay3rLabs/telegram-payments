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
        // Sanity check: amount must be greater than zero
        if payment.amount.is_zero() {
            return;
        }

        // check if there is an existing entry with this denom and combine them
        if let Some(i) = self.payments.iter().position(|p| p.denom == payment.denom) {
            self.payments[i].amount += payment.amount;
        } else {
            // Insert in alphabetically sorted position
            let insert_pos = self
                .payments
                .iter()
                .position(|p| p.denom > payment.denom)
                .unwrap_or(self.payments.len());
            self.payments.insert(insert_pos, payment);
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Uint256;

    #[test]
    fn test_one_pending_payment() {
        let mut pending = PendingPayments::default();
        pending.add_payment(Coin {
            amount: Uint256::from(100u128),
            denom: "uusd".to_string(),
        });
        assert_eq!(
            pending.balance(),
            vec![Coin {
                amount: Uint256::from(100u128),
                denom: "uusd".to_string()
            }]
        );
    }

    #[test]
    fn test_duplicate_denom() {
        let mut pending = PendingPayments::default();
        pending.add_payment(Coin {
            amount: Uint256::from(100u128),
            denom: "uusd".to_string(),
        });
        pending.add_payment(Coin {
            amount: Uint256::from(200u128),
            denom: "uusd".to_string(),
        });
        assert_eq!(
            pending.balance(),
            vec![Coin {
                amount: Uint256::from(300u128),
                denom: "uusd".to_string()
            }]
        );
    }

    #[test]
    fn test_sorting_denoms() {
        let mut pending = PendingPayments::default();
        pending.add_payment(Coin {
            amount: Uint256::from(100u128),
            denom: "uusd".to_string(),
        });
        pending.add_payment(Coin {
            amount: Uint256::from(200u128),
            denom: "ntrn".to_string(),
        });
        assert_eq!(
            pending.balance(),
            vec![
                Coin {
                    amount: Uint256::from(200u128),
                    denom: "ntrn".to_string()
                },
                Coin {
                    amount: Uint256::from(100u128),
                    denom: "uusd".to_string()
                }
            ]
        );
    }
}
