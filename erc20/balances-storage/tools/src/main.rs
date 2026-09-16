#[cfg(not(target_arch = "wasm32"))]
fn main() {
    match erc20_balances_storage_tools::cli::run() {
        Ok(true) => (),
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
