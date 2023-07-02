use rocket::fairing::AdHoc;

pub mod jwt;

pub fn attach() -> AdHoc {
    AdHoc::on_ignite("Manage services", |rocket| async {
        rocket.manage(jwt::new())
    })
}
