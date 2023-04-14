use airtifex_core::user::AccountType;

use bitflags::bitflags;

bitflags! {
    struct AccountTypeFlag: u32 {
       const ADMIN = 0b00000001;
       const USER = 0b00000010;
       const SERVICE = 0b00000100;
    }
}

pub struct AclBuilder {
    account_types: AccountTypeFlag,
}

impl Default for AclBuilder {
    fn default() -> Self {
        Self {
            account_types: AccountTypeFlag::ADMIN,
        }
    }
}

impl AclBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(self) -> Self {
        self.with_user().with_service().with_admin()
    }

    pub fn without_admin(mut self) -> Self {
        self.account_types -= AccountTypeFlag::ADMIN;
        self
    }

    pub fn with_admin(mut self) -> Self {
        self.account_types |= AccountTypeFlag::ADMIN;
        self
    }

    pub fn with_user(mut self) -> Self {
        self.account_types |= AccountTypeFlag::USER;
        self
    }

    pub fn with_service(mut self) -> Self {
        self.account_types |= AccountTypeFlag::SERVICE;
        self
    }

    pub fn build(self) -> Acl {
        Acl {
            account_types: self.account_types,
        }
    }
}

pub struct Acl {
    account_types: AccountTypeFlag,
}

impl Acl {
    pub fn builder() -> AclBuilder {
        AclBuilder::default()
    }

    pub fn has_account_type(&self, _type: AccountType) -> bool {
        self.account_types
            .contains(AccountTypeFlag::from_bits_truncate((_type as i32) as u32))
    }
}
