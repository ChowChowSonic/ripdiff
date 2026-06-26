fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn farewell(name: &str) -> String {
    format!("Goodbye, {}!", name)
}

fn main() {
    let msg = greet("World");
    println!("{}", msg);
    let bye = farewell("World");
    println!("{}", bye);
}
