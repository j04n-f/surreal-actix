#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone)]
pub struct CreateAccount {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}
