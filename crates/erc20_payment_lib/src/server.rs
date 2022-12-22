use crate::db::operations::*;
use crate::eth::get_eth_addr_from_secret;
use crate::runtime::SharedState;
use crate::setup::PaymentSetup;
use actix_web::web::Data;
use actix_web::{web, HttpRequest, Responder};
use serde_json::json;
use sqlx::Connection;
use sqlx::SqliteConnection;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ServerData {
    pub shared_state: Arc<Mutex<SharedState>>,
    pub db_connection: Arc<Mutex<SqliteConnection>>,
    pub payment_setup: PaymentSetup,
}

macro_rules! return_on_error {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(err) => {
                return web::Json(json!({
                    "error": err.to_string()
                }))
            },
        }
    }
}

pub async fn tx_details(data: Data<Box<ServerData>>, req: HttpRequest) -> impl Responder {
    let tx_id = req
        .match_info()
        .get("tx_id")
        .map(|tx_id| i64::from_str(tx_id).ok())
        .unwrap_or(None);

    let tx_id = match tx_id {
        Some(tx_id) => tx_id,
        None => return web::Json(json!({"error": "failed to parse tx_id"})),
    };

    let tx = {
        let mut db_conn = data.db_connection.lock().await;
        match get_transaction(&mut db_conn, tx_id).await {
            Ok(allowances) => allowances,
            Err(err) => {
                return web::Json(json!({
                    "error": err.to_string()
                }));
                //return format!("Error getting allowances: {:?}", err);
            }
        }
    };

    /*
    let transfers = {
        let mut db_conn = data.db_connection.lock().await;
        match get_token_transfers_by_tx(&mut db_conn, tx_id).await {
            Ok(allowances) => allowances,
            Err(err) => {
                return web::Json(json!({
                    "error": err.to_string()
                }))
            }
        }
    };*/
    /*let json_transfers = transfers
    .iter()
    .map(|transfer| {
        json!({
            "id": transfer.id,
            "chain_id": transfer.chain_id,
            "tx_id": transfer.tx_id,
            "from": transfer.from_addr,
            "receiver": transfer.receiver_addr,
            "token": transfer.token_addr,
            "amount": transfer.token_amount,
            "fee_paid": transfer.fee_paid,
        })
    })
    .collect::<Vec<_>>();*/

    web::Json(json!({
        "tx": tx,
    }))
}

pub async fn allowances(data: Data<Box<ServerData>>, _req: HttpRequest) -> impl Responder {
    let mut my_data = data.shared_state.lock().await;
    my_data.inserted += 1;

    let allowances = {
        let mut db_conn = data.db_connection.lock().await;
        match get_all_allowances(&mut db_conn).await {
            Ok(allowances) => allowances,
            Err(err) => {
                return web::Json(json!({
                    "error": err.to_string()
                }));
                //return format!("Error getting allowances: {:?}", err);
            }
        }
    };

    let json_allowances = allowances
        .iter()
        .map(|allowance| {
            json!({
                "id": allowance.id,
                "chain_id": allowance.chain_id,
                "tx_id": allowance.tx_id,
                "owner": allowance.owner,
                "token": allowance.token_addr,
                "spender": allowance.spender,
                "amount": allowance.allowance,
                "confirm_date": allowance.confirm_date,
            })
        })
        .collect::<Vec<_>>();

    web::Json(json!({
        "allowances": json_allowances,
    }))
}

pub async fn transactions_count(data: Data<Box<ServerData>>, _req: HttpRequest) -> impl Responder {
    let queued_tx_count = {
        let mut db_conn = data.db_connection.lock().await;
        return_on_error!(get_transaction_count(&mut db_conn, Some(TRANSACTION_FILTER_QUEUED)).await)
    };
    let done_tx_count = {
        let mut db_conn = data.db_connection.lock().await;
        return_on_error!(get_transaction_count(&mut db_conn, Some(TRANSACTION_FILTER_DONE)).await)
    };

    let queued_transfer_count = {
        let mut db_conn = data.db_connection.lock().await;
        return_on_error!(get_transfer_count(&mut db_conn, Some(TRANSFER_FILTER_QUEUED)).await)
    };
    let processed_transfer_count = {
        let mut db_conn = data.db_connection.lock().await;
        return_on_error!(get_transfer_count(&mut db_conn, Some(TRANSFER_FILTER_PROCESSING)).await)
    };
    let done_transfer_count = {
        let mut db_conn = data.db_connection.lock().await;
        return_on_error!(get_transfer_count(&mut db_conn, Some(TRANSFER_FILTER_DONE)).await)
    };

    web::Json(json!({
        "transfersQueued": queued_transfer_count,
        "transfersProcessing": processed_transfer_count,
        "transfersDone": done_transfer_count,
        "txQueued": queued_tx_count,
        "txDone": done_tx_count,
    }))
}

pub async fn config_endpoint(data: Data<Box<ServerData>>) -> impl Responder {
    let mut payment_setup = data.payment_setup.clone();
    payment_setup.secret_keys = vec![];

    web::Json(json!({
        "config": payment_setup,
    }))
}

pub async fn transactions(data: Data<Box<ServerData>>, _req: HttpRequest) -> impl Responder {
    //todo: add limits
    let txs = {
        let mut db_conn = data.db_connection.lock().await;
        return_on_error!(get_transactions(&mut db_conn, None, None, None).await)
    };
    web::Json(json!({
        "txs": txs,
    }))
}

pub async fn skip_pending_operation(
    data: Data<Box<ServerData>>,
    req: HttpRequest,
) -> impl Responder {
    let tx_id = req
        .match_info()
        .get("tx_id")
        .map(|tx_id| i64::from_str(tx_id).ok())
        .unwrap_or(None);
    if let Some(tx_id) = tx_id {
        if data.shared_state.lock().await.skip_tx(tx_id) {
            web::Json(json!({
                "success": "true",
            }))
        } else {
            web::Json(json!({
                "error": "Tx not found",
            }))
        }
    } else {
        web::Json(json!({
            "error": "failed to parse tx_id",
        }))
    }
}

pub async fn transactions_next(data: Data<Box<ServerData>>, req: HttpRequest) -> impl Responder {
    let limit = req
        .match_info()
        .get("count")
        .map(|tx_id| i64::from_str(tx_id).ok())
        .unwrap_or(Some(10));

    let txs = {
        let mut db_conn = data.db_connection.lock().await;
        return_on_error!(
            get_transactions(
                &mut db_conn,
                Some(TRANSACTION_FILTER_QUEUED),
                limit,
                Some(TRANSACTION_ORDER_BY_CREATE_DATE)
            )
            .await
        )
    };
    web::Json(json!({
        "txs": txs,
    }))
}
pub async fn transactions_current(
    data: Data<Box<ServerData>>,
    _req: HttpRequest,
) -> impl Responder {
    let txs = {
        let mut db_conn = data.db_connection.lock().await;
        return_on_error!(
            get_transactions(
                &mut db_conn,
                Some(TRANSACTION_FILTER_PROCESSING),
                None,
                Some(TRANSACTION_ORDER_BY_CREATE_DATE)
            )
            .await
        )
    };
    web::Json(json!({
        "txs": txs,
    }))
}

pub async fn transactions_last_processed(
    data: Data<Box<ServerData>>,
    req: HttpRequest,
) -> impl Responder {
    let limit = req
        .match_info()
        .get("count")
        .map(|tx_id| i64::from_str(tx_id).ok())
        .unwrap_or(Some(10));

    let txs = {
        let mut db_conn = data.db_connection.lock().await;
        return_on_error!(
            get_transactions(
                &mut db_conn,
                Some(TRANSACTION_FILTER_DONE),
                limit,
                Some(TRANSACTION_ORDER_BY_FIRST_PROCESSED_DATE_DESC)
            )
            .await
        )
    };
    web::Json(json!({
        "txs": txs,
    }))
}

pub async fn transactions_feed(data: Data<Box<ServerData>>, req: HttpRequest) -> impl Responder {
    let limit_prev = req
        .match_info()
        .get("prev")
        .map(|tx_id| i64::from_str(tx_id).ok())
        .unwrap_or(Some(10));
    let limit_next = req
        .match_info()
        .get("next")
        .map(|tx_id| i64::from_str(tx_id).ok())
        .unwrap_or(Some(10));
    let mut txs = {
        let mut db_conn = data.db_connection.lock().await;
        let mut db_transaction = return_on_error!(db_conn.begin().await);
        let mut txs = return_on_error!(
            get_transactions(
                &mut db_transaction,
                Some(TRANSACTION_FILTER_DONE),
                limit_prev,
                Some(TRANSACTION_ORDER_BY_FIRST_PROCESSED_DATE_DESC)
            )
            .await
        );
        let txs_current = return_on_error!(
            get_transactions(
                &mut db_transaction,
                Some(TRANSACTION_FILTER_PROCESSING),
                None,
                Some(TRANSACTION_ORDER_BY_CREATE_DATE)
            )
            .await
        );
        let tx_next = return_on_error!(
            get_transactions(
                &mut db_transaction,
                Some(TRANSACTION_FILTER_QUEUED),
                limit_next,
                Some(TRANSACTION_ORDER_BY_CREATE_DATE)
            )
            .await
        );
        return_on_error!(db_transaction.commit().await);
        //join transactions
        txs.reverse();
        txs.extend(txs_current);
        txs.extend(tx_next);
        txs
    };

    let current_tx = data.shared_state.lock().await.current_tx_info.clone();
    for tx in txs.iter_mut() {
        if let Some(tx_info) = current_tx.get(&tx.id) {
            tx.engine_error = tx_info.error.clone();
            tx.engine_message = Some(tx_info.message.clone());
        }
    }

    web::Json(json!({
        "txs": txs,
        "current": current_tx,
    }))
}

pub async fn transfers(data: Data<Box<ServerData>>, req: HttpRequest) -> impl Responder {
    let tx_id = req
        .match_info()
        .get("tx_id")
        .map(|tx_id| i64::from_str(tx_id).ok())
        .unwrap_or(None);

    //let my_data = data.shared_state.lock().await;

    let transfers = {
        let mut db_conn = data.db_connection.lock().await;
        if let Some(tx_id) = tx_id {
            match get_token_transfers_by_tx(&mut db_conn, tx_id).await {
                Ok(allowances) => allowances,
                Err(err) => {
                    return web::Json(json!({
                        "error": err.to_string()
                    }))
                }
            }
        } else {
            match get_all_token_transfers(&mut db_conn, None).await {
                Ok(allowances) => allowances,
                Err(err) => {
                    return web::Json(json!({
                        "error": err.to_string()
                    }))
                }
            }
        }
    };

    /*
        let json_transfers = transfers
            .iter()
            .map(|transfer| {
                json!({
                    "id": transfer.id,
                    "chain_id": transfer.chain_id,
                    "tx_id": transfer.tx_id,
                    "from": transfer.from_addr,
                    "receiver": transfer.receiver_addr,
                    "token": transfer.token_addr,
                    "amount": transfer.token_amount,
                    "fee_paid": transfer.fee_paid,
                })
            })
            .collect::<Vec<_>>();
    */
    web::Json(json!({
        "transfers": transfers,
    }))
}

pub async fn accounts(data: Data<Box<ServerData>>, _req: HttpRequest) -> impl Responder {
    //let name = req.match_info().get("name").unwrap_or("World");
    //let mut my_data = data.shared_state.lock().await;
    //my_data.inserted += 1;

    let public_addr = data
        .payment_setup
        .secret_keys
        .iter()
        .map(|sk| get_eth_addr_from_secret(sk).to_string());

    json!({
        "public_addr": public_addr.collect::<Vec<String>>()
    })
    .to_string()
}

pub async fn greet(_data: Data<Box<ServerData>>, req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    //let mut my_data = data.shared_state.lock().await;
    //my_data.inserted += 1;

    format!("Hello {}!", name)
}
