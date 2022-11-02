
CREATE TABLE "tx" (
    id	                INTEGER     NOT NULL,
    from_addr           TEXT        NOT NULL,
    to_addr             TEXT        NOT NULL,
    chain_id            INTEGER     NOT NULL,
    gas_limit           INTEGER     NOT NULL,
    max_fee_per_gas     TEXT        NOT NULL,
    priority_fee        TEXT        NOT NULL,
    val                 TEXT        NOT NULL,
    nonce               TEXT        NULL,
    call_data           TEXT        NOT NULL,
    created_date        DATETIME    NOT NULL,
    tx_hash             TEXT        NULL,
    signed_raw_data     TEXT        NULL,
    signed_date         DATETIIME   NULL,
    broadcast_date      DATETIIME   NULL,
    confirmed_date      DATETIIME   NULL,
    block_number        INTEGER     NULL,
    chain_status        INTEGER     NULL,
    fee_paid            TEXT        NULL,
    PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE "token_transfer" (
    id	                INTEGER     NOT NULL,
    from_addr           TEXT        NOT NULL,
    receiver_addr       TEXT        NOT NULL,
    chain_id            INTEGER     NOT NULL,
    token_addr          TEXT        NULL,
    token_amount        TEXT        NOT NULL,
    tx_id               INTEGER     NULL,
    fee_paid            TEXT        NULL,
    PRIMARY KEY("id" AUTOINCREMENT)
    CONSTRAINT "fk_token_transfer_tx" FOREIGN KEY("tx_id") REFERENCES "tx"("id")
);


