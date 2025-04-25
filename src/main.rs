
fn main() {
    let string = "BOOBS//FACE";
    let parts:Vec<&str> = string.split("/").collect();
    println!("{}", parts.len());
}

