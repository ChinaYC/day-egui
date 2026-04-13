fn main() {
    let mut path = dirs::home_dir().unwrap();
    path.push(".leetcode_automation");
    println!("{:?}", path);
}