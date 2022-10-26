CREATE TABLE "tx" (
    unique_id	        TEXT,
    from_addr           TEXT,
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
    fee_paid            TEXT        NULL
);