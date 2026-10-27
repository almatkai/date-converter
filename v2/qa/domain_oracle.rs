use std::io::Read;
fn main() {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).unwrap();
    let tests: Vec<serde_json::Value> = serde_json::from_str(&s).unwrap();
    let output = tests
        .into_iter()
        .map(|t| {
            let p = date_converter_v2::converter::Preferences::read(&t["preferences"]).unwrap();
            let r = date_converter_v2::converter::results(
                t["text"].as_str().unwrap(),
                t["now"].as_i64().unwrap(),
                t["tz"].as_str().unwrap().parse().unwrap(),
                t["target"]
                    .as_str()
                    .unwrap_or(t["tz"].as_str().unwrap())
                    .parse()
                    .unwrap(),
                &p,
            );
            match r {
                Ok(rows) => serde_json::json!({"rows":rows}),
                Err(e) => serde_json::json!({"error":e}),
            }
        })
        .collect::<Vec<_>>();
    println!("{}", serde_json::to_string(&output).unwrap());
}
