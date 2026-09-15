use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub requests: &'a u64,
    pub grainpit_urls: &'a u32,
}

#[derive(Template)]
#[template(path = "auth.html")]
pub struct AuthTemplate<'a> {
    pub redirect: &'a String,
    pub message: &'a String,
}

#[derive(Template)]
#[template(path = "completed_auth.html")]
pub struct CompletedAuthTemplate<'a> {
    pub key: &'a String,
    pub redirect: &'a String,
}

#[derive(Template)]
#[template(path = "account.html")]
pub struct AccountTemplate<'a> {
    pub username: &'a String,
    pub urls: &'a String,
    pub admin: &'a bool,
}

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminTemplate {}
