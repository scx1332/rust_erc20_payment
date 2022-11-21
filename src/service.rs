use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use crate::db::operations::{
    find_allowance, get_allowance_by_tx, get_pending_token_transfers, get_token_transfers_by_tx,
    get_transactions_being_processed, insert_allowance, insert_tx, update_allowance,
    update_token_transfer, update_tx,
};
use crate::error::{AllowanceRequest, PaymentError};
use crate::model::{Allowance, TokenTransfer, Web3TransactionDao};
use crate::multi::check_allowance;
use crate::process::{process_transaction, ProcessTransactionResult};
use crate::transaction::{
    create_erc20_approve, create_erc20_transfer, create_erc20_transfer_multi, create_eth_transfer,
};
use crate::utils::ConversionError;

use crate::setup::PaymentSetup;
use sqlx::{Connection, SqliteConnection};
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

pub async fn process_allowance(
    conn: &mut SqliteConnection,
    payment_setup: &PaymentSetup,
    allowance_request: &AllowanceRequest,
) -> Result<u32, PaymentError> {
    let minimum_allowance: U256 = U256::max_value() / U256::from(2);
    let chain_setup = payment_setup.get_chain_setup(allowance_request.chain_id)?;
    let web3 = payment_setup.get_provider(allowance_request.chain_id)?;
    let max_fee_per_gas = chain_setup.max_fee_per_gas;
    let priority_fee = chain_setup.priority_fee;

    let mut db_allowance = find_allowance(
        conn,
        &allowance_request.owner,
        &allowance_request.token_addr,
        &allowance_request.spender_addr,
        allowance_request.chain_id,
    )
    .await?;

    let allowance = match db_allowance.as_mut() {
        Some(db_allowance) => match db_allowance.confirm_date {
            Some(_) => {
                log::debug!("Allowance already confirmed from db");
                U256::from_dec_str(&db_allowance.allowance)?
            }
            None => {
                log::debug!("Allowance not confirmed in db, check on chain");
                let allowance = check_allowance(
                    web3,
                    Address::from_str(&allowance_request.owner)?,
                    Address::from_str(&allowance_request.token_addr)?,
                    Address::from_str(&allowance_request.spender_addr)?,
                )
                .await?;
                if allowance > minimum_allowance {
                    log::debug!(
                        "Allowance found on chain, update db_allowance with id {}",
                        db_allowance.id
                    );
                    db_allowance.confirm_date = Some(chrono::Utc::now());
                    update_allowance(conn, db_allowance).await?;
                }
                allowance
            }
        },
        None => {
            log::debug!("No db entry, check allowance on chain");
            let allowance = check_allowance(
                web3,
                Address::from_str(&allowance_request.owner)?,
                Address::from_str(&allowance_request.token_addr)?,
                Address::from_str(&allowance_request.spender_addr)?,
            )
            .await?;
            if allowance > minimum_allowance {
                log::debug!("Allowance found on chain, add entry to db");
                let db_allowance = Allowance {
                    id: 0,
                    owner: allowance_request.owner.clone(),
                    token_addr: allowance_request.token_addr.clone(),
                    spender: allowance_request.spender_addr.clone(),
                    chain_id: allowance_request.chain_id,
                    tx_id: None,
                    allowance: allowance.to_string(),
                    confirm_date: Some(chrono::Utc::now()),
                    fee_paid: None,
                    error: None,
                };
                //allowance is confirmed on web3, update db
                insert_allowance(conn, &db_allowance).await?;
            }
            allowance
        }
    };

    if allowance < minimum_allowance {
        let mut allowance = Allowance {
            id: 0,
            owner: allowance_request.owner.clone(),
            token_addr: allowance_request.token_addr.clone(),
            spender: allowance_request.spender_addr.clone(),
            allowance: U256::max_value().to_string(),
            chain_id: allowance_request.chain_id,
            tx_id: None,
            fee_paid: None,
            confirm_date: None,
            error: None,
        };

        let approve_tx = create_erc20_approve(
            Address::from_str(&allowance_request.owner)?,
            Address::from_str(&allowance_request.token_addr)?,
            Address::from_str(&allowance_request.spender_addr)?,
            allowance_request.chain_id as u64,
            1000,
            max_fee_per_gas,
            priority_fee,
        )?;
        let mut db_transaction = conn.begin().await?;
        let web3_tx_dao = insert_tx(&mut db_transaction, &approve_tx).await?;
        allowance.tx_id = Some(web3_tx_dao.id);
        insert_allowance(&mut db_transaction, &allowance).await?;

        db_transaction.commit().await?;

        return Ok(1);
    }
    Ok(0)
}

type TokenTransferMap = HashMap<TokenTransferKey, Vec<TokenTransfer>>;

pub async fn gather_transactions_pre(
    conn: &mut SqliteConnection,
) -> Result<TokenTransferMap, PaymentError> {
    let mut transfer_map = TokenTransferMap::new();

    let mut token_transfers = get_pending_token_transfers(conn).await?;

    for f in token_transfers.iter_mut() {
        match Address::from_str(&f.from_addr) {
            Ok(from_addr) => {
                if from_addr == Address::zero() {
                    f.error = Some("from_addr is zero".to_string());
                    update_token_transfer(conn, f).await?;
                    continue;
                }
            }
            Err(_err) => {
                f.error = Some("Invalid from address".to_string());
                update_token_transfer(conn, f).await?;
                continue;
            }
        }
        match Address::from_str(&f.receiver_addr) {
            Ok(rec_address) => {
                if rec_address == Address::zero() {
                    f.error = Some("receiver_addr is zero".to_string());
                    update_token_transfer(conn, f).await?;
                    continue;
                }
            }
            Err(_err) => {
                f.error = Some("Invalid receiver address".to_string());
                update_token_transfer(conn, f).await?;
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

#[derive(Debug, Clone)]
pub struct TokenTransferMultiOrder {
    receiver: Address,
    token_transfers: Vec<TokenTransfer>,
}

pub async fn gather_transactions_batch_multi(
    conn: &mut SqliteConnection,
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
                    &format!("{:#x}", multi_contract_address),
                    token_transfer.chain_id,
                )
                .await?;

                let mut allowance_not_met = false;
                match db_allowance {
                    Some(db_allowance) => match db_allowance.confirm_date {
                        Some(_) => {
                            let allowance = U256::from_dec_str(&db_allowance.allowance)?;
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
                    return Err(PaymentError::NoAllowanceFound(AllowanceRequest {
                        owner: token_transfer.from_addr.clone(),
                        token_addr: token_addr.clone(),
                        spender_addr: format!("{:#x}", multi_contract_address),
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
                    sum += U256::from_dec_str(&token_transfer.token_amount)?;
                }
                erc20_to.push(token_t.receiver);
                erc20_amounts.push(sum);
            }

            let web3tx = match erc20_to.len() {
                0 => {
                    return Ok(0);
                }
                1 => create_erc20_transfer(
                    Address::from_str(&token_transfer.from_addr)?,
                    Address::from_str(token_addr)?,
                    erc20_to[0],
                    erc20_amounts[0],
                    token_transfer.chain_id as u64,
                    1000,
                    max_fee_per_gas,
                    priority_fee,
                )?,
                _ => create_erc20_transfer_multi(
                    Address::from_str(&token_transfer.from_addr)?,
                    chain_setup.multi_contract_address.unwrap(),
                    erc20_to,
                    erc20_amounts,
                    token_transfer.chain_id as u64,
                    1000,
                    max_fee_per_gas,
                    priority_fee,
                    false,
                )?,
            };
            let mut db_transaction = conn.begin().await?;
            let web3_tx_dao = insert_tx(&mut db_transaction, &web3tx).await?;

            for token_t in &mut *smaller_order {
                for token_transfer in &mut token_t.token_transfers {
                    token_transfer.tx_id = Some(web3_tx_dao.id);
                    update_token_transfer(&mut db_transaction, token_transfer).await?;
                }
            }
            db_transaction.commit().await?;
        }
    } else {
        return Err(PaymentError::OtherError(
            "Not implemented for multi".to_string(),
        ));
    };

    Ok(1)
}

pub async fn gather_transactions_batch(
    conn: &mut SqliteConnection,
    payment_setup: &PaymentSetup,
    token_transfers: &mut [TokenTransfer],
    token_transfer: &TokenTransferKey,
) -> Result<u32, PaymentError> {
    let mut sum = U256::zero();
    for token_transfer in token_transfers.iter() {
        sum += U256::from_dec_str(&token_transfer.token_amount)?;
    }

    let chain_setup = payment_setup.get_chain_setup(token_transfer.chain_id)?;

    let max_fee_per_gas = chain_setup.max_fee_per_gas;
    let priority_fee = chain_setup.priority_fee;

    log::debug!("Processing token transfer {:?}", token_transfer);
    let web3tx = if let Some(token_addr) = token_transfer.token_addr.as_ref() {
        create_erc20_transfer(
            Address::from_str(&token_transfer.from_addr)?,
            Address::from_str(token_addr)?,
            Address::from_str(&token_transfer.receiver_addr)?,
            sum,
            token_transfer.chain_id as u64,
            1000,
            max_fee_per_gas,
            priority_fee,
        )?
    } else {
        create_eth_transfer(
            Address::from_str(&token_transfer.from_addr)?,
            Address::from_str(&token_transfer.receiver_addr)?,
            token_transfer.chain_id as u64,
            1000,
            max_fee_per_gas,
            priority_fee,
            sum,
        )
    };
    let mut db_transaction = conn.begin().await?;
    let web3_tx_dao = insert_tx(&mut db_transaction, &web3tx).await?;
    for token_transfer in token_transfers.iter_mut() {
        token_transfer.tx_id = Some(web3_tx_dao.id);
        update_token_transfer(&mut db_transaction, token_transfer).await?;
    }
    db_transaction.commit().await?;
    Ok(1)
}

pub async fn gather_transactions_post(
    conn: &mut SqliteConnection,
    payment_setup: &PaymentSetup,
    token_transfer_map: &mut TokenTransferMap,
) -> Result<u32, PaymentError> {
    let mut inserted_tx_count = 0;

    let mut sorted_order = BTreeMap::<i64, TokenTransferKey>::new();

    for pair in token_transfer_map.iter() {
        let token_transfers = pair.1;
        let token_transfer = pair.0;
        let min_id = token_transfers.iter().map(|f| f.id).min().ok_or_else(|| {
            PaymentError::OtherError("Failed algorithm when searching min".to_string())
        })?;
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
                let token_transfers =
                    token_transfer_map.get_mut(token_transfer).ok_or_else(|| {
                        PaymentError::OtherError("Failed algorithm when getting key".to_string())
                    })?;

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
                        match e {
                            PaymentError::NoAllowanceFound(allowance_request) => {
                                //pass allowance error up
                                return Err(PaymentError::NoAllowanceFound(allowance_request));
                            }
                            _ => {
                                //mark other errors in db to not process these failed transfers again
                                for token_transfer in token_transfers {
                                    token_transfer.error =
                                        Some("Error in gathering transactions".to_string());
                                    update_token_transfer(conn, token_transfer).await?;
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
                        receiver: Address::from_str(&key.1.receiver_addr)?,
                        token_transfers: opt,
                    });
                }
                None => {
                    multi_key_map.insert(
                        multi_key,
                        vec![TokenTransferMultiOrder {
                            token_transfers: opt,
                            receiver: Address::from_str(&key.1.receiver_addr)?,
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
                    match e {
                        PaymentError::NoAllowanceFound(allowance_request) => {
                            //pass allowance error up
                            return Err(PaymentError::NoAllowanceFound(allowance_request));
                        }
                        _ => {
                            //mark other errors in db to not process these failed transfers again
                            for multi in token_transfers {
                                for token_transfer in multi.token_transfers {
                                    let mut tt = token_transfer.clone();
                                    tt.error = Some("Error in gathering transactions".to_string());
                                    update_token_transfer(conn, &tt).await?;
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

pub async fn update_token_transfer_result(
    conn: &mut SqliteConnection,
    tx: &mut Web3TransactionDao,
    process_t_res: ProcessTransactionResult,
) -> Result<(), PaymentError> {
    match process_t_res {
        ProcessTransactionResult::Confirmed => {
            tx.processing = 0;

            let mut db_transaction = conn.begin().await?;
            let token_transfers = get_token_transfers_by_tx(&mut db_transaction, tx.id).await?;
            let token_transfers_count = U256::from(token_transfers.len() as u64);
            for mut token_transfer in token_transfers {
                if let Some(fee_paid) = tx.fee_paid.clone() {
                    let val = U256::from_dec_str(&fee_paid)
                        .map_err(|_err| ConversionError::from("failed to parse fee paid".into()))?;
                    let val2 = val / token_transfers_count;
                    token_transfer.fee_paid = Some(val2.to_string());
                } else {
                    token_transfer.fee_paid = None;
                }
                update_token_transfer(&mut db_transaction, &token_transfer).await?;
            }
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        }
        ProcessTransactionResult::NeedRetry(err) => {
            tx.processing = 0;

            let mut db_transaction = conn.begin().await?;
            let token_transfers = get_token_transfers_by_tx(&mut db_transaction, tx.id).await?;
            for mut token_transfer in token_transfers {
                token_transfer.fee_paid = Some("0".to_string());
                token_transfer.error = Some(err.clone());
                update_token_transfer(&mut db_transaction, &token_transfer).await?;
            }
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        }
        ProcessTransactionResult::InternalError(err) => {
            tx.processing = 0;

            let mut db_transaction = conn.begin().await?;
            let token_transfers = get_token_transfers_by_tx(&mut db_transaction, tx.id).await?;
            for mut token_transfer in token_transfers {
                token_transfer.fee_paid = Some("0".to_string());
                token_transfer.error = Some(err.clone());
                update_token_transfer(&mut db_transaction, &token_transfer).await?;
            }
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        }
        ProcessTransactionResult::Unknown => {
            tx.processing = 1;
            update_tx(conn, tx).await?;
        }
    }
    Ok(())
}

pub async fn update_approve_result(
    conn: &mut SqliteConnection,
    tx: &mut Web3TransactionDao,
    process_t_res: ProcessTransactionResult,
) -> Result<(), PaymentError> {
    match process_t_res {
        ProcessTransactionResult::Confirmed => {
            tx.processing = 0;

            let mut db_transaction = conn.begin().await?;
            let mut allowance = get_allowance_by_tx(&mut db_transaction, tx.id).await?;
            allowance.fee_paid = tx.fee_paid.clone();
            update_allowance(&mut db_transaction, &allowance).await?;
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        }
        ProcessTransactionResult::NeedRetry(err) => {
            tx.processing = 0;
            let mut db_transaction = conn.begin().await?;
            let mut allowance = get_allowance_by_tx(&mut db_transaction, tx.id).await?;
            allowance.fee_paid = Some("0".to_string());
            allowance.error = Some(err.clone());
            update_allowance(&mut db_transaction, &allowance).await?;
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        }
        ProcessTransactionResult::InternalError(err) => {
            tx.processing = 0;
            let mut db_transaction = conn.begin().await?;
            let mut allowance = get_allowance_by_tx(&mut db_transaction, tx.id).await?;
            allowance.fee_paid = Some("0".to_string());
            allowance.error = Some(err.clone());
            update_allowance(&mut db_transaction, &allowance).await?;
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        }
        ProcessTransactionResult::Unknown => {
            tx.processing = 1;
            update_tx(conn, tx).await?;
        }
    }
    Ok(())
}

pub async fn update_tx_result(
    conn: &mut SqliteConnection,
    tx: &mut Web3TransactionDao,
    process_t_res: ProcessTransactionResult,
) -> Result<(), PaymentError> {
    match process_t_res {
        ProcessTransactionResult::Confirmed => {
            tx.processing = 0;
            update_tx(conn, tx).await?;
        }
        ProcessTransactionResult::NeedRetry(_err) => {
            tx.processing = 0;
            tx.error = Some("Need retry".to_string());
            update_tx(conn, tx).await?;
        }
        ProcessTransactionResult::InternalError(err) => {
            tx.processing = 0;
            tx.error = Some(err.clone());
            update_tx(conn, tx).await?;
        }
        ProcessTransactionResult::Unknown => {
            tx.processing = 1;
            update_tx(conn, tx).await?;
        }
    }
    Ok(())
}

pub async fn process_transactions(
    conn: &mut SqliteConnection,
    payment_setup: &PaymentSetup,
) -> Result<(), PaymentError> {
    loop {
        let mut transactions = get_transactions_being_processed(conn).await?;

        //TODO - This loop is getting only first element, fix code so only one transaction is taken from db
        #[allow(clippy::never_loop)]
        for tx in &mut transactions {
            let process_t_res = match process_transaction(conn, tx, payment_setup, false).await {
                Ok(process_result) => process_result,
                Err(err) => match err {
                    PaymentError::TransactionFailedError(err) => {
                        ProcessTransactionResult::InternalError(err)
                    }
                    _ => {
                        return Err(err);
                    }
                },
            };
            if tx.method.starts_with("MULTI.golemTransfer")
                || tx.method == "ERC20.transfer"
                || tx.method == "transfer"
            {
                log::debug!("Updating token transfer result");
                update_token_transfer_result(conn, tx, process_t_res).await?;
            } else if tx.method == "ERC20.approve" {
                log::debug!("Updating token approve result");
                update_approve_result(conn, tx, process_t_res).await?;
            } else {
                update_tx_result(conn, tx, process_t_res).await?;
            }
            //process only one transaction at once
            break;
        }
        if transactions.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
    Ok(())
}

pub async fn service_loop(conn: &mut SqliteConnection, payment_setup: PaymentSetup) {
    let process_transactions_interval = 5;
    let gather_transactions_interval = 20;
    let mut last_update_time1 =
        chrono::Utc::now() - chrono::Duration::seconds(process_transactions_interval);
    let mut last_update_time2 =
        chrono::Utc::now() - chrono::Duration::seconds(gather_transactions_interval);

    let mut process_tx_needed = true;
    let mut process_tx_instantly = true;
    loop {
        let current_time = chrono::Utc::now();
        if current_time < last_update_time1 {
            //handle case when system time changed
            last_update_time1 = current_time;
        }

        if process_tx_instantly
            || (process_tx_needed
                && current_time
                    > last_update_time1 + chrono::Duration::seconds(process_transactions_interval))
        {
            process_tx_instantly = false;
            if payment_setup.generate_tx_only {
                log::warn!("Skipping processing transactions...");
                process_tx_needed = false;
            } else {
                match process_transactions(conn, &payment_setup).await {
                    Ok(_) => {
                        //all pending transactions processed
                        process_tx_needed = false;
                    }
                    Err(e) => {
                        log::error!("Error in process transactions: {}", e);
                    }
                };
            }
            last_update_time1 = current_time;
        }

        if current_time
            > last_update_time2 + chrono::Duration::seconds(gather_transactions_interval)
        {
            log::debug!("Gathering transfers...");
            let mut token_transfer_map = match gather_transactions_pre(conn).await {
                Ok(token_transfer_map) => token_transfer_map,
                Err(e) => {
                    log::error!("Error in gather transactions, driver will be stuck, Fix DB to continue {:?}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            };

            match gather_transactions_post(conn, &payment_setup, &mut token_transfer_map).await {
                Ok(count) => {
                    if count > 0 {
                        process_tx_needed = true;
                    }
                }
                Err(e) => {
                    match &e {
                        PaymentError::NoAllowanceFound(allowance_request) => {
                            log::error!("No allowance found for: {:?}", allowance_request);
                            match process_allowance(conn, &payment_setup, allowance_request).await {
                                Ok(_) => {
                                    //process transaction instantly
                                    process_tx_needed = true;
                                    process_tx_instantly = true;
                                    continue;
                                    //process_tx_needed = true;
                                }
                                Err(e) => {
                                    log::error!("Error in process allowance: {}", e);
                                }
                            }
                        }
                        _ => {
                            log::error!("Error in gather transactions: {}", e);
                        }
                    }
                    //if error happened, we should check if partial transfers were inserted
                    process_tx_needed = true;
                    log::error!("Error in gather transactions: {}", e);
                }
            };
            last_update_time2 = current_time;
            if payment_setup.finish_when_done && !process_tx_needed {
                log::info!("No more work to do, exiting...");
                break;
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
