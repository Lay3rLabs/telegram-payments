use cosmwasm_schema::write_api;
 
use tg_contract_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
 
fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        // 👇 only add those entries if you use the sudo/migrate entry point
        sudo: SudoMsg,
        migrate: MigrateMsg,
    }
}
