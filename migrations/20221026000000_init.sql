
CREATE TABLE "tx" (
    id	                TEXT        NOT NULL,
    from_addr           TEXT        NOT NULL,
    to_addr             TEXT        NOT NULL,
    chain_id            INT         NOT NULL,
    gas_limit           INT         NOT NULL,
    max_fee_per_gas     TEXT        NOT NULL,
    priority_fee        TEXT        NOT NULL,
    val                 TEXT        NOT NULL,
    nonce               TEXT        NOT NULL,
    call_data           TEXT        NOT NULL,
    created_date        DATETIME    NOT NULL,
    tx_hash             TEXT        NULL,
    signed_raw_data     TEXT        NULL,
    signed_date         DATETIIME   NULL,
    broadcast_date      DATETIIME   NULL,
    confirmed_date      DATETIIME   NULL,
    block_number        INT         NULL,
    chain_status        INT         NULL,
    fee_paid            TEXT        NULL,
    PRIMARY KEY("id")
);

CREATE TABLE "token_transfer" (
    id	                TEXT        NOT NULL,
    from_addr           TEXT        NOT NULL,
    receiver_addr       TEXT        NOT NULL,
    chain_id            INT         NOT NULL,
    token_addr          TEXT        NOT NULL,
    token_amount        TEXT        NOT NULL,
    tx_id               TEXT        NULL,
    fee_paid            TEXT        NULL,
    PRIMARY KEY("id")
    CONSTRAINT "fk_token_transfer_tx" FOREIGN KEY("tx_id") REFERENCES "tx"("id")
);


