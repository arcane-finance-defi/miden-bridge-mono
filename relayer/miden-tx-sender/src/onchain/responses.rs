use miden_objects::{Word, account::{Account, AuthSecretKey}};

pub struct CreatedTokenAccount {
    account: Account,
    seed: Word,
    auth_secret_key: AuthSecretKey,
}

impl CreatedTokenAccount {
    pub fn new(account: Account, seed: Word, auth_secret_key: AuthSecretKey) -> Self {
        Self { account, seed, auth_secret_key }
    }

    pub fn auth_secret_key(&self) -> AuthSecretKey {
        self.auth_secret_key.clone()
    }

    pub fn account(&self) -> Account {
        self.account.clone()
    }

    pub fn seed(&self) -> Word {
        self.seed.clone()
    }
}
