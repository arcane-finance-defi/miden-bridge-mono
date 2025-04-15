CREATE TABLE assets_info
(
    origin_network  INT  NOT NULL,
    origin_address  TEXT NOT NULL,
    miden_faucet_id BLOB NOT NULL,
    PRIMARY KEY (origin_network, origin_address)
);
