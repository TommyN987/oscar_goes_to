// `sqlx::migrate!` embeds the migrations at compile time, so a new migration must trigger a rebuild.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
