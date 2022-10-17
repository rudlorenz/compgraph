use compgraph::{create_input, mul, sum};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let a = create_input("a");
    let b = create_input("b");
    let c = create_input("c");

    let rslt = sum(a.clone(), mul(b.clone(), c.clone()));

    log::debug!("{a:?}");
    log::debug!("{b:?}");
    log::debug!("{c:?}");

    a.set(10.);
    b.set(50.);
    c.set(30.);

    log::info!("{rslt}");

    log::info!("compute : {}", rslt.compute());
    log::info!("compute : {}", rslt.compute());

    a.set(20.);
    log::info!("compute : {}", rslt.compute());
}
