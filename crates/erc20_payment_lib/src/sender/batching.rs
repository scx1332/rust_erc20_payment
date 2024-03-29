use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use crate::db::model::*;
use crate::db::ops::*;
use crate::error::{AllowanceRequest, ErrorBag, PaymentError};

use crate::transaction::{create_erc20_transfer, create_erc20_transfer_multi, create_eth_transfer};

use crate::error::CustomError;
use crate::setup::PaymentSetup;
use crate::{err_create, err_custom_create, err_from};

use sqlx::SqlitePool;

use web3::types::{Address, U256};

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct TokenTransferKey {
    pub from_addr: String,
    pub receiver_addr: String,
    pub chain_id: i64,
    pub token_addr: Option<String>,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct TokenTransferMultiKey {
    pub from_addr: String,
    pub chain_id: i64,
    pub token_addr: Option<String>,
}

type TokenTransferMap = HashMap<TokenTransferKey, Vec<TokenTransferDao>>;

#[derive(Debug, Clone)]
pub struct TokenTransferMultiOrder {
    receiver: Address,
    token_transfers: Vec<TokenTransferDao>,
}

pub async fn gather_transactions_pre(
    conn: &SqlitePool,
    _payment_setup: &PaymentSetup,
) -> Result<TokenTransferMap, PaymentError> {
    let mut transfer_map = TokenTransferMap::new();

    let mut token_transfers = get_pending_token_transfers(conn)
        .await
        .map_err(err_from!())?;

    for f in token_transfers.iter_mut() {
        match Address::from_str(&f.from_addr) {
            Ok(from_addr) => {
                if from_addr == Address::zero() {
                    f.error = Some("from_addr is zero".to_string());
                    update_token_transfer(conn, f).await.map_err(err_from!())?;
                    continue;
                }
                //@TODO: check if from_addr is in a wallet
                /*
                if from_addr != payment_setup.pub_address {
                    f.error = Some("no from_addr in wallet".to_string());
                    update_token_transfer(conn, f).await.map_err(err_from!())?;
                    continue;
                }*/
            }
            Err(_err) => {
                f.error = Some("Invalid from address".to_string());
                update_token_transfer(conn, f).await.map_err(err_from!())?;
                continue;
            }
        }
        match Address::from_str(&f.receiver_addr) {
            Ok(rec_address) => {
                if rec_address == Address::zero() {
                    f.error = Some("receiver_addr is zero".to_string());
                    update_token_transfer(conn, f).await.map_err(err_from!())?;
                    continue;
                }
            }
            Err(_err) => {
                f.error = Some("Invalid receiver address".to_string());
                update_token_transfer(conn, f).await.map_err(err_from!())?;
                continue;
            }
        }

        //group transactions
        let key = TokenTransferKey {
            from_addr: f.from_addr.clone(),
            receiver_addr: f.receiver_addr.clone(),
            chain_id: f.chain_id,
            token_addr: f.token_addr.clone(),
        };
        match transfer_map.get_mut(&key) {
            Some(v) => {
                v.push(f.clone());
            }
            None => {
                transfer_map.insert(key, vec![f.clone()]);
            }
        }
    }
    Ok(transfer_map)
}

pub async fn gather_transactions_batch_multi(
    conn: &SqlitePool,
    payment_setup: &PaymentSetup,
    multi_order_vector: &mut [TokenTransferMultiOrder],
    token_transfer: &TokenTransferMultiKey,
) -> Result<u32, PaymentError> {
    let chain_setup = payment_setup.get_chain_setup(token_transfer.chain_id)?;

    let max_fee_per_gas = chain_setup.max_fee_per_gas;
    let priority_fee = chain_setup.priority_fee;

    let max_per_batch = chain_setup.multi_contract_max_at_once;
    log::debug!("Processing token transfer {:?}", token_transfer);
    if let Some(token_addr) = token_transfer.token_addr.as_ref() {
        if !payment_setup.skip_multi_contract_check {
            if let Some(multi_contract_address) = chain_setup.multi_contract_address.as_ref() {
                //this is some arbitrary number.
                let minimum_allowance: U256 = U256::max_value() / U256::from(2);

                let db_allowance = find_allowance(
                    conn,
                    &token_transfer.from_addr,
                    token_addr,
                    &format!("{multi_contract_address:#x}"),
                    token_transfer.chain_id,
                )
                .await
                .map_err(err_from!())?;

                let mut allowance_not_met = false;
                match db_allowance {
                    Some(db_allowance) => match db_allowance.confirm_date {
                        Some(_) => {
                            let allowance =
                                U256::from_dec_str(&db_allowance.allowance).map_err(err_from!())?;
                            if allowance < minimum_allowance {
                                log::debug!(
                                    "Allowance already confirmed from db, but it is too small"
                                );
                                allowance_not_met = true;
                            } else {
                                log::debug!("Allowance confirmed from db");
                            }
                        }
                        None => {
                            log::debug!("Allowance request found, but not confirmed");
                            allowance_not_met = true;
                        }
                    },
                    None => {
                        log::debug!("Allowance not found in db");
                        allowance_not_met = true;
                    }
                };
                if allowance_not_met {
                    return Err(err_create!(AllowanceRequest {
                        owner: token_transfer.from_addr.clone(),
                        token_addr: token_addr.clone(),
                        spender_addr: format!("{multi_contract_address:#x}"),
                        chain_id: token_transfer.chain_id,
                        amount: U256::max_value(),
                    }));
                }
            }
        }

        let split_orders = multi_order_vector
            .chunks_mut(max_per_batch)
            .collect::<Vec<_>>();

        for smaller_order in split_orders {
            let mut erc20_to = Vec::with_capacity(smaller_order.len());
            let mut erc20_amounts = Vec::with_capacity(smaller_order.len());
            for token_t in &mut *smaller_order {
                let mut sum = U256::zero();
                for token_transfer in &token_t.token_transfers {
                    sum += U256::from_dec_str(&token_transfer.token_amount).map_err(err_from!())?;
                }
                erc20_to.push(token_t.receiver);
                erc20_amounts.push(sum);
            }

            let web3tx = match erc20_to.len() {
                0 => {
                    return Ok(0);
                }
                1 => {
                    log::info!(
                        "Inserting transaction stub for ERC20 transfer to: {:?}",
                        erc20_to[0]
                    );

                    create_erc20_transfer(
                        Address::from_str(&token_transfer.from_addr).map_err(err_from!())?,
                        Address::from_str(token_addr).map_err(err_from!())?,
                        erc20_to[0],
                        erc20_amounts[0],
                        token_transfer.chain_id as u64,
                        None,
                        max_fee_per_gas,
                        priority_fee,
                    )?
                }
                _ => {
                    log::info!("Inserting transaction stub for ERC20 multi transfer contract: {:?} for {} distinct transfers", chain_setup.multi_contract_address.unwrap(), erc20_to.len());

                    create_erc20_transfer_multi(
                        Address::from_str(&token_transfer.from_addr).map_err(err_from!())?,
                        chain_setup.multi_contract_address.unwrap(),
                        erc20_to,
                        erc20_amounts,
                        token_transfer.chain_id as u64,
                        None,
                        max_fee_per_gas,
                        priority_fee,
                        false,
                    )?
                }
            };
            let mut db_transaction = conn.begin().await.map_err(err_from!())?;
            let web3_tx_dao = insert_tx(&mut db_transaction, &web3tx)
                .await
                .map_err(err_from!())?;

            for token_t in &mut *smaller_order {
                for token_transfer in &mut token_t.token_transfers {
                    token_transfer.tx_id = Some(web3_tx_dao.id);
                    update_token_transfer(&mut db_transaction, token_transfer)
                        .await
                        .map_err(err_from!())?;
                }
            }
            db_transaction.commit().await.map_err(err_from!())?;
        }
    } else {
        return Err(err_custom_create!("Not implemented for multi"));
    };

    Ok(1)
}

pub async fn gather_transactions_batch(
    conn: &SqlitePool,
    payment_setup: &PaymentSetup,
    token_transfers: &mut [TokenTransferDao],
    token_transfer: &TokenTransferKey,
) -> Result<u32, PaymentError> {
    let mut sum = U256::zero();
    for token_transfer in token_transfers.iter() {
        sum += U256::from_dec_str(&token_transfer.token_amount).map_err(err_from!())?;
    }

    let chain_setup = payment_setup.get_chain_setup(token_transfer.chain_id)?;

    let max_fee_per_gas = chain_setup.max_fee_per_gas;
    let priority_fee = chain_setup.priority_fee;

    log::debug!("Processing token transfer {:?}", token_transfer);
    let web3tx = if let Some(token_addr) = token_transfer.token_addr.as_ref() {
        create_erc20_transfer(
            Address::from_str(&token_transfer.from_addr).map_err(err_from!())?,
            Address::from_str(token_addr).map_err(err_from!())?,
            Address::from_str(&token_transfer.receiver_addr).map_err(err_from!())?,
            sum,
            token_transfer.chain_id as u64,
            None,
            max_fee_per_gas,
            priority_fee,
        )?
    } else {
        create_eth_transfer(
            Address::from_str(&token_transfer.from_addr).map_err(err_from!())?,
            Address::from_str(&token_transfer.receiver_addr).map_err(err_from!())?,
            token_transfer.chain_id as u64,
            None,
            max_fee_per_gas,
            priority_fee,
            sum,
        )
    };
    let mut db_transaction = conn.begin().await.map_err(err_from!())?;
    let web3_tx_dao = insert_tx(&mut db_transaction, &web3tx)
        .await
        .map_err(err_from!())?;
    for token_transfer in token_transfers.iter_mut() {
        token_transfer.tx_id = Some(web3_tx_dao.id);
        update_token_transfer(&mut db_transaction, token_transfer)
            .await
            .map_err(err_from!())?;
    }
    db_transaction.commit().await.map_err(err_from!())?;
    Ok(1)
}

pub async fn gather_transactions_post(
    conn: &SqlitePool,
    payment_setup: &PaymentSetup,
    token_transfer_map: &mut TokenTransferMap,
) -> Result<u32, PaymentError> {
    let mut inserted_tx_count = 0;

    let mut sorted_order = BTreeMap::<i64, TokenTransferKey>::new();

    for pair in token_transfer_map.iter() {
        let token_transfers = pair.1;
        let token_transfer = pair.0;
        let min_id = token_transfers
            .iter()
            .map(|f| f.id)
            .min()
            .ok_or_else(|| err_custom_create!("Failed algorithm when searching min"))?;
        sorted_order.insert(min_id, token_transfer.clone());
    }
    let use_multi = true;
    if use_multi {
        let mut multi_key_map =
            HashMap::<TokenTransferMultiKey, Vec<TokenTransferMultiOrder>>::new();
        for key in &sorted_order {
            let multi_key = TokenTransferMultiKey {
                from_addr: key.1.from_addr.clone(),
                chain_id: key.1.chain_id,
                token_addr: key.1.token_addr.clone(),
            };
            if multi_key.token_addr.is_none() {
                let token_transfer = key.1;
                let token_transfers = token_transfer_map
                    .get_mut(token_transfer)
                    .ok_or_else(|| err_custom_create!("Failed algorithm when getting key"))?;

                //sum of transfers
                match gather_transactions_batch(
                    conn,
                    payment_setup,
                    token_transfers,
                    token_transfer,
                )
                .await
                {
                    Ok(_) => {
                        inserted_tx_count += 1;
                    }
                    Err(e) => {
                        match &e.inner {
                            ErrorBag::NoAllowanceFound(_allowance_request) => {
                                //pass allowance error up
                                return Err(e);
                            }
                            _ => {
                                //mark other errors in db to not process these failed transfers again
                                for token_transfer in token_transfers {
                                    token_transfer.error =
                                        Some("Error in gathering transactions".to_string());
                                    update_token_transfer(conn, token_transfer)
                                        .await
                                        .map_err(err_from!())?;
                                }
                                log::error!("Failed to gather transactions: {:?}", e);
                            }
                        }
                    }
                }
                inserted_tx_count += 1;
                continue;
            }

            //todo - fix unnecessary clone here
            let opt = token_transfer_map.get(key.1).unwrap().clone();
            token_transfer_map.remove(key.1);

            match multi_key_map.get_mut(&multi_key) {
                Some(v) => {
                    v.push(TokenTransferMultiOrder {
                        receiver: Address::from_str(&key.1.receiver_addr).map_err(err_from!())?,
                        token_transfers: opt,
                    });
                }
                None => {
                    multi_key_map.insert(
                        multi_key,
                        vec![TokenTransferMultiOrder {
                            token_transfers: opt,
                            receiver: Address::from_str(&key.1.receiver_addr)
                                .map_err(err_from!())?,
                        }],
                    );
                }
            }
        }
        for key in multi_key_map {
            let token_transfer = key.0;
            let mut token_transfers = key.1.clone();
            //todo fix clones
            match gather_transactions_batch_multi(
                conn,
                payment_setup,
                &mut token_transfers,
                &token_transfer,
            )
            .await
            {
                Ok(_) => {
                    inserted_tx_count += 1;
                }
                Err(e) => {
                    match &e.inner {
                        ErrorBag::NoAllowanceFound(_allowance_request) => {
                            //pass allowance error up
                            return Err(e);
                        }
                        _ => {
                            //mark other errors in db to not process these failed transfers again
                            for multi in token_transfers {
                                for token_transfer in multi.token_transfers {
                                    let mut tt = token_transfer.clone();
                                    tt.error = Some("Error in gathering transactions".to_string());
                                    update_token_transfer(conn, &tt)
                                        .await
                                        .map_err(err_from!())?;
                                }
                            }
                            log::error!("Failed to gather transactions: {:?}", e);
                        }
                    }
                }
            }
            inserted_tx_count += 1;
        }
    }

    Ok(inserted_tx_count)
}
